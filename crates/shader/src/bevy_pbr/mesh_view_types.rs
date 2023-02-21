use spirv_std::glam::{Mat4, UVec3, UVec4, Vec3, Vec4};

#[allow(unused_imports)]
use spirv_std::num_traits::Float;

use crate::saturate::Saturate;

use super::{
    clustered_forward::CLUSTER_COUNT_SIZE,
    pbr_lighting::{fd_burley, get_distance_attenuation, specular},
};

#[derive(Default, Copy, Clone, PartialEq)]
#[repr(C)]
pub struct View {
    pub view_proj: Mat4,
    pub inverse_view_proj: Mat4,
    pub view: Mat4,
    pub inverse_view: Mat4,
    pub projection: Mat4,
    pub inverse_projection: Mat4,
    pub world_position: Vec3,
    // viewport(x_origin, y_origin, width, height)
    pub viewport: Vec4,
}

#[derive(Default, Copy, Clone, PartialEq)]
#[repr(C)]
pub struct PointLight {
    // For point lights: the lower-right 2x2 values of the projection matrix [2][2] [2][3] [3][2] [3][3]
    // For spot lights: the direction (x,z), spot_scale and spot_offset
    pub light_custom_data: Vec4,
    pub color_inverse_square_range: Vec4,
    pub position_radius: Vec4,
    // 'flags' is a bit field indicating various options. u32 is 32 bits so we have up to 32 options.
    pub flags: u32,
    pub shadow_depth_bias: f32,
    pub shadow_normal_bias: f32,
    pub spot_light_tan_angle: f32,
}

impl PointLight {
    pub fn point_light(
        &self,
        world_position: Vec3,
        roughness: f32,
        n_dot_v: f32,
        n: Vec3,
        v: Vec3,
        r: Vec3,
        f0: Vec3,
        diffuse_color: Vec3,
    ) -> Vec3 {
        let light_to_frag = self.position_radius.truncate() - world_position;
        let distance_square = light_to_frag.dot(light_to_frag);
        let range_attenuation =
            get_distance_attenuation(distance_square, self.color_inverse_square_range.w);

        // Specular.
        // Representative Point Area Lights.
        // see http://blog.selfshadow.com/publications/s2013-shading-course/karis/s2013_pbs_epic_notes_v2.pdf p14-16
        let a = roughness;
        let center_to_ray = light_to_frag.dot(r) * r - light_to_frag;
        let closest_point = light_to_frag
            + center_to_ray
                * (self.position_radius.w * center_to_ray.dot(center_to_ray).sqrt().recip())
                    .saturate();
        let l_spec_length_inverse = closest_point.dot(closest_point).sqrt().recip();
        let normalization_factor =
            a / (a + (self.position_radius.w * 0.5 * l_spec_length_inverse)).saturate();
        let specular_intensity = normalization_factor * normalization_factor;

        let l: Vec3 = closest_point * l_spec_length_inverse; // ().normalize() equivalent?
        let h: Vec3 = (l + v).normalize();
        let nol: f32 = n.dot(l).saturate();
        let noh: f32 = n.dot(h).saturate();
        let loh: f32 = l.dot(h).saturate();

        let specular_light = specular(f0, roughness, h, n_dot_v, nol, noh, loh, specular_intensity);

        // Diffuse.
        // Comes after specular since its NoL is used in the lighting equation.
        let l = light_to_frag.normalize();
        let h = (l + v).normalize();
        let nol = n.dot(l).saturate();
        let _noh = n.dot(h).saturate();
        let loh = l.dot(h).saturate();

        let diffuse = diffuse_color * fd_burley(roughness, n_dot_v, nol, loh);

        // See https://google.github.io/filament/Filament.html#mjx-eqn-pointLightLuminanceEquation
        // Lout = f(v,l) Φ / { 4 π d^2 }⟨n⋅l⟩
        // where
        // f(v,l) = (f_d(v,l) + f_r(v,l)) * light_color
        // Φ is luminous power in lumens
        // our rangeAttentuation = 1 / d^2 multiplied with an attenuation factor for smoothing at the edge of the non-physical maximum light radius

        // For a point light, luminous intensity, I, in lumens per steradian is given by:
        // I = Φ / 4 π
        // The derivation of this can be seen here: https://google.github.io/filament/Filament.html#mjx-eqn-pointLightLuminousPower

        // NOTE: light.color.rgb is premultiplied with light.intensity / 4 π (which would be the luminous intensity) on the CPU

        // TODO compensate for energy loss https://google.github.io/filament/Filament.html#materialsystem/improvingthebrdfs/energylossinspecularreflectance

        (diffuse + specular_light)
            * self.color_inverse_square_range.truncate()
            * (range_attenuation * nol)
    }

    pub fn spot_light(
        &self,
        world_position: Vec3,
        roughness: f32,
        n_dot_v: f32,
        n: Vec3,
        v: Vec3,
        r: Vec3,
        f0: Vec3,
        diffuse_color: Vec3,
    ) -> Vec3 {
        // reuse the point light calculations
        let point_light = self.point_light(
            world_position,
            roughness,
            n_dot_v,
            n,
            v,
            r,
            f0,
            diffuse_color,
        );

        // reconstruct spot dir from x/z and y-direction flag
        let mut spot_dir = Vec3::new(self.light_custom_data.x, 0.0, self.light_custom_data.y);
        spot_dir.y = (0.0_f32.max(1.0 - spot_dir.x * spot_dir.x - spot_dir.z * spot_dir.z)).sqrt();
        if (self.flags & POINT_LIGHT_FLAGS_SPOT_LIGHT_Y_NEGATIVE) != 0 {
            spot_dir.y = -spot_dir.y;
        }
        let light_to_frag = self.position_radius.truncate() - world_position;

        // calculate attenuation based on filament formula https://google.github.io/filament/Filament.html#listing_glslpunctuallight
        // spot_scale and spot_offset have been precomputed
        // note we normalize here to get "l" from the filament listing. spot_dir is already normalized
        let cd = -spot_dir.dot(light_to_frag.normalize());
        let attenuation = (cd * self.light_custom_data.z + self.light_custom_data.w).saturate();
        let spot_attenuation = attenuation * attenuation;

        point_light * spot_attenuation
    }
}

pub const POINT_LIGHT_FLAGS_SHADOWS_ENABLED_BIT: u32 = 1;
pub const POINT_LIGHT_FLAGS_SPOT_LIGHT_Y_NEGATIVE: u32 = 2;

#[derive(Default, Copy, Clone, PartialEq)]
#[repr(C)]
pub struct DirectionalLight {
    pub view_projection: Mat4,
    pub color: Vec4,
    pub direction_to_light: Vec3,
    // 'flags' is a bit field indicating various options. u32 is 32 bits so we have up to 32 options.
    pub flags: u32,
    pub shadow_depth_bias: f32,
    pub shadow_normal_bias: f32,
}

impl DirectionalLight {
    pub fn directional_light(
        &self,
        roughness: f32,
        n_dot_v: f32,
        normal: Vec3,
        view: Vec3,
        f0: Vec3,
        diffuse_color: Vec3,
    ) -> Vec3 {
        let incident_light = self.direction_to_light;

        let half_vector = (incident_light + view).normalize();
        let nol = (normal.dot(incident_light)).saturate();
        let noh = (normal.dot(half_vector)).saturate();
        let loh = (incident_light.dot(half_vector)).saturate();

        let diffuse = diffuse_color * fd_burley(roughness, n_dot_v, nol, loh);
        let specular_intensity = 1.0;
        let specular_light = specular(
            f0,
            roughness,
            half_vector,
            n_dot_v,
            nol,
            noh,
            loh,
            specular_intensity,
        );

        (specular_light + diffuse) * self.color.truncate() * nol
    }
}

pub const DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT: u32 = 1;

#[derive(Default, Copy, Clone, PartialEq)]
#[repr(C)]
pub struct Lights {
    // NOTE: this array size must be kept in sync with the constants defined in bevy_pbr/src/render/light.rs
    pub directional_lights: [DirectionalLight; 10],
    pub ambient_color: Vec4,
    // x/y/z dimensions and n_clusters in w
    pub cluster_dimensions: UVec4,
    // xy are vec2<f32>(cluster_dimensions.xy) / vec2<f32>(view.width, view.height)
    //
    // For perspective projections:
    // z is cluster_dimensions.z / log(far / near)
    // w is cluster_dimensions.z * log(near) / log(far / near)
    //
    // For orthographic projections:
    // NOTE: near and far are +ve but -z is infront of the camera
    // z is -near
    // w is cluster_dimensions.z / (-far - -near)
    pub cluster_factors: Vec4,
    pub n_directional_lights: u32,
    pub spot_light_shadowmap_offset: i32,
}

#[cfg(feature = "NO_STORAGE_BUFFERS_SUPPORT")]
#[derive(Copy, Clone, PartialEq)]
#[repr(C)]
pub struct PointLights {
    pub data: [PointLight; 256],
}

#[cfg(feature = "NO_STORAGE_BUFFERS_SUPPORT")]
#[derive(Copy, Clone, PartialEq)]
#[repr(C)]
pub struct ClusterLightIndexLists {
    // each u32 contains 4 u8 indices into the PointLights array
    pub data: [UVec4; 1024],
}

impl ClusterLightIndexLists {
    pub fn get_light_id(&self, index: u32) -> u32 {
        #[cfg(feature = "NO_STORAGE_BUFFERS_SUPPORT")]
        {
            // The index is correct but in cluster_light_index_lists we pack 4 u8s into a u32
            // This means the index into cluster_light_index_lists is index / 4
            let v = self.data[(index >> 4) as usize];
            let indices = match ((index >> 2) & ((1 << 2) - 1)) as usize {
                0 => v.x,
                1 => v.y,
                2 => v.z,
                3 => v.w,
                _ => panic!(),
            };
            // And index % 4 gives the sub-index of the u8 within the u32 so we shift by 8 * sub-index
            (indices >> (8 * (index & ((1 << 2) - 1)))) & ((1 << 8) - 1)
        }

        #[cfg(not(feature = "NO_STORAGE_BUFFERS_SUPPORT"))]
        {
            unsafe { *self.data.index(index as usize) }
        }
    }
}

#[cfg(feature = "NO_STORAGE_BUFFERS_SUPPORT")]
#[derive(Clone, PartialEq)]
#[repr(C)]
pub struct ClusterOffsetsAndCounts {
    // each u32 contains a 24-bit index into ClusterLightIndexLists in the high 24 bits
    // and an 8-bit count of the number of lights in the low 8 bits
    pub data: [UVec4; 1024],
}

impl ClusterOffsetsAndCounts {
    pub fn unpack(&self, cluster_index: u32) -> UVec3 {
        #[cfg(feature = "NO_STORAGE_BUFFERS_SUPPORT")]
        {
            let v = self.data[(cluster_index >> 2) as usize];
            let i = cluster_index & ((1 << 2) - 1);
            let offset_and_counts = match i {
                0 => v.x,
                1 => v.y,
                2 => v.z,
                3 => v.w,
                _ => panic!(),
            };
            //  [ 31     ..     18 | 17      ..      9 | 8       ..     0 ]
            //  [      offset      | point light count | spot light count ]
            UVec3::new(
                (offset_and_counts >> (CLUSTER_COUNT_SIZE * 2))
                    & ((1 << (32 - (CLUSTER_COUNT_SIZE * 2))) - 1),
                (offset_and_counts >> CLUSTER_COUNT_SIZE) & ((1 << CLUSTER_COUNT_SIZE) - 1),
                offset_and_counts & ((1 << CLUSTER_COUNT_SIZE) - 1),
            )
        }

        #[cfg(not(feature = "NO_STORAGE_BUFFERS_SUPPORT"))]
        {
            unsafe { self.data.index(cluster_index as usize) }.truncate()
        }
    }
}

#[cfg(not(feature = "NO_STORAGE_BUFFERS_SUPPORT"))]
#[repr(C)]
pub struct PointLights {
    pub data: spirv_std::RuntimeArray<PointLight>,
}

#[cfg(not(feature = "NO_STORAGE_BUFFERS_SUPPORT"))]
#[repr(C)]
pub struct ClusterLightIndexLists {
    pub data: RuntimeArray<u32>,
}

#[cfg(not(feature = "NO_STORAGE_BUFFERS_SUPPORT"))]
#[repr(C)]
pub struct ClusterOffsetsAndCounts {
    pub data: RuntimeArray<UVec4>,
}

#[repr(C)]
pub struct Globals {
    // The time since startup in seconds
    // Wraps to 0 after 1 hour.
    pub time: f32,
    // The delta time since the previous frame in seconds
    pub delta_time: f32,
    // Frame count since the start of the app.
    // It wraps to zero when it reaches the maximum value of a u32.
    pub frame_count: u32,

    #[cfg(feature = "SIXTEEN_BYTE_ALIGNMENT")]
    // WebGL2 structs must be 16 byte aligned.
    _wasm_padding: f32,
}
