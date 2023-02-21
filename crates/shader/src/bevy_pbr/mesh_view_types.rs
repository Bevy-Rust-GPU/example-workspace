use spirv_std::{
    arch::unsigned_min,
    glam::{Mat3, Mat4, UVec3, UVec4, Vec2, Vec3, Vec4},
    Sampler,
};

#[allow(unused_imports)]
use spirv_std::num_traits::Float;

use crate::saturate::Saturate;

use super::{
    clustered_forward::CLUSTER_COUNT_SIZE,
    mesh_functions::mesh_position_local_to_world,
    pbr_lighting::{fd_burley, get_distance_attenuation, specular},
    shadows::{DirectionalShadowTextures, PointShadowTextures},
};

const NATURAL_LOG_BASE: f32 = 2.718281828459;

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

impl View {
    // NOTE: Correctly calculates the view vector depending on whether
    // the projection is orthographic or perspective.
    pub fn calculate_view(&self, world_position: Vec4, is_orthographic: bool) -> Vec3 {
        if is_orthographic {
            // Orthographic view vector
            Vec3::new(
                self.view_proj.x_axis.z,
                self.view_proj.y_axis.z,
                self.view_proj.z_axis.z,
            )
            .normalize()
        } else {
            // Only valid for a perpective projection
            (self.world_position - world_position.truncate()).normalize()
        }
    }

    pub fn mesh_position_world_to_clip(&self, world_position: Vec4) -> Vec4 {
        self.view_proj * world_position
    }

    // NOTE: The intermediate world_position assignment is important
    // for precision purposes when using the 'equals' depth comparison
    // function.
    pub fn mesh_position_local_to_clip(&self, model: Mat4, vertex_position: Vec4) -> Vec4 {
        let world_position = mesh_position_local_to_world(model, vertex_position);
        self.mesh_position_world_to_clip(world_position)
    }
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

impl Lights {
    pub fn fetch_directional_shadow(
        &self,
        directional_shadow_textures: &DirectionalShadowTextures,
        directional_shadow_textures_sampler: &Sampler,
        light_id: u32,
        frag_position: Vec4,
        surface_normal: Vec3,
    ) -> f32 {
        let light = &self.directional_lights[light_id as usize];

        // The normal bias is scaled to the texel size.
        let normal_offset = light.shadow_normal_bias * surface_normal;
        let depth_offset = light.shadow_depth_bias * light.direction_to_light;
        let offset_position =
            (frag_position.truncate() + normal_offset + depth_offset).extend(frag_position.w);

        let offset_position_clip = light.view_projection * offset_position;
        if offset_position_clip.w <= 0.0 {
            return 1.0;
        }
        let offset_position_ndc = offset_position_clip.truncate() / offset_position_clip.w;
        // No shadow outside the orthographic projection volume
        if (offset_position_ndc.x < -1.0 || offset_position_ndc.y < -1.0)
            || offset_position_ndc.z < 0.0
            || (offset_position_ndc.x > 1.0
                || offset_position_ndc.y > 1.0
                || offset_position_ndc.z > 1.0)
        {
            return 1.0;
        }

        // compute texture coordinates for shadow lookup, compensating for the Y-flip difference
        // between the NDC and texture coordinates
        let flip_correction = Vec2::new(0.5, -0.5);
        let light_local = offset_position_ndc.truncate() * flip_correction + Vec2::new(0.5, 0.5);

        let depth = offset_position_ndc.z;
        // do the lookup, using HW PCF and comparison
        // NOTE: Due to non-uniform control flow above, we must use the level variant of the texture
        // sampler to avoid use of implicit derivatives causing possible undefined behavior.
        #[cfg(feature = "NO_ARRAY_TEXTURES_SUPPORT")]
        {
            directional_shadow_textures.sample_depth_reference(
                *directional_shadow_textures_sampler,
                light_local.extend(0.0),
                depth,
            )
        }

        #[cfg(not(feature = "NO_ARRAY_TEXTURES_SUPPORT"))]
        {
            directional_shadow_textures.sample_depth_reference_by_lod(
                *directional_shadow_textures_sampler,
                light_local.extend(0.0),
                depth,
                light_id as f32,
            )
        }
    }

    // NOTE: Keep in sync with bevy_pbr/src/light.rs
    pub fn view_z_to_z_slice(&self, view_z: f32, is_orthographic: bool) -> u32 {
        let z_slice = if is_orthographic {
            // NOTE: view_z is correct in the orthographic case
            ((view_z - self.cluster_factors.z) * self.cluster_factors.w).floor() as u32
        } else {
            // NOTE: had to use -view_z to make it positive else log(negative) is nan
            ((-view_z).log(NATURAL_LOG_BASE) * self.cluster_factors.z - self.cluster_factors.w
                + 1.0) as u32
        };
        // NOTE: We use min as we may limit the far z plane used for clustering to be closeer than
        // the furthest thing being drawn. This means that we need to limit to the maximum cluster.
        unsigned_min(z_slice, self.cluster_dimensions.z - 1)
    }

    pub fn fragment_cluster_index(
        &self,
        view: &View,
        frag_coord: Vec2,
        view_z: f32,
        is_orthographic: bool,
    ) -> u32 {
        let xy = ((frag_coord - view.viewport.truncate().truncate())
            * self.cluster_factors.truncate().truncate())
        .floor()
        .as_uvec2();
        let z_slice = self.view_z_to_z_slice(view_z, is_orthographic);
        // NOTE: Restricting cluster index to avoid undefined behavior when accessing uniform buffer
        // arrays based on the cluster index.
        unsigned_min(
            (xy.y * self.cluster_dimensions.x + xy.x) * self.cluster_dimensions.z + z_slice,
            self.cluster_dimensions.w - 1,
        )
    }
}

#[cfg(feature = "NO_STORAGE_BUFFERS_SUPPORT")]
#[derive(Copy, Clone, PartialEq)]
#[repr(C)]
pub struct PointLights {
    pub data: [PointLight; 256],
}

impl PointLights {
    pub fn fetch_point_shadow(
        &self,
        point_shadow_textures: &PointShadowTextures,
        point_shadow_textures_sampler: &Sampler,
        light_id: u32,
        frag_position: Vec4,
        surface_normal: Vec3,
    ) -> f32 {
        #[cfg(feature = "NO_STORAGE_BUFFERS_SUPPORT")]
        let light = &self.data[light_id as usize];

        #[cfg(not(feature = "NO_STORAGE_BUFFERS_SUPPORT"))]
        let light = unsafe { self.data.index(light_id as usize) };

        // because the shadow maps align with the axes and the frustum planes are at 45 degrees
        // we can get the worldspace depth by taking the largest absolute axis
        let surface_to_light = light.position_radius.truncate() - frag_position.truncate();
        let surface_to_light_abs = surface_to_light.abs();
        let distance_to_light = surface_to_light_abs
            .x
            .max(surface_to_light_abs.y.max(surface_to_light_abs.z));

        // The normal bias here is already scaled by the texel size at 1 world unit from the light.
        // The texel size increases proportionally with distance from the light so multiplying by
        // distance to light scales the normal bias to the texel size at the fragment distance.
        let normal_offset = light.shadow_normal_bias * distance_to_light * surface_normal;
        let depth_offset = light.shadow_depth_bias * surface_to_light.normalize();
        let offset_position = frag_position.truncate() + normal_offset + depth_offset;

        // similar largest-absolute-axis trick as above, but now with the offset fragment position
        let frag_ls = light.position_radius.truncate() - offset_position;
        let abs_position_ls = frag_ls.abs();
        let major_axis_magnitude = abs_position_ls
            .x
            .max(abs_position_ls.y.max(abs_position_ls.z));

        // NOTE: These simplifications come from multiplying:
        // projection * vec4(0, 0, -major_axis_magnitude, 1.0)
        // and keeping only the terms that have any impact on the depth.
        // Projection-agnostic approach:
        let zw = -major_axis_magnitude * light.light_custom_data.truncate().truncate()
            + Vec2::new(light.light_custom_data.z, light.light_custom_data.w);
        let depth = zw.x / zw.y;

        // do the lookup, using HW PCF and comparison
        // NOTE: Due to the non-uniform control flow above, we must use the Level variant of
        // textureSampleCompare to avoid undefined behaviour due to some of the fragments in
        // a quad (2x2 fragments) being processed not being sampled, and this messing with
        // mip-mapping functionality. The shadow maps have no mipmaps so Level just samples
        // from LOD 0.
        #[cfg(feature = "NO_ARRAY_TEXTURES_SUPPORT")]
        {
            point_shadow_textures.sample_depth_reference(
                *point_shadow_textures_sampler,
                frag_ls.extend(1.0),
                depth,
            )
        }

        #[cfg(not(feature = "NO_ARRAY_TEXTURES_SUPPORT"))]
        {
            point_shadow_textures.sample_depth_reference_by_lod(
                *point_shadow_textures_sampler,
                frag_ls.extend(1.0),
                depth,
                light_id as f32,
            )
        }
    }

    pub fn fetch_spot_shadow(
        &self,
        lights: &Lights,
        directional_shadow_textures: &DirectionalShadowTextures,
        directional_shadow_textures_sampler: &Sampler,
        light_id: u32,
        frag_position: Vec4,
        surface_normal: Vec3,
    ) -> f32 {
        #[cfg(feature = "NO_STORAGE_BUFFERS_SUPPORT")]
        let light = &self.data[light_id as usize];

        #[cfg(not(feature = "NO_STORAGE_BUFFERS_SUPPORT"))]
        let light = unsafe { self.data.index(light_id as usize) };

        let surface_to_light = light.position_radius.truncate() - frag_position.truncate();

        // construct the light view matrix
        let mut spot_dir = Vec3::new(light.light_custom_data.x, 0.0, light.light_custom_data.y);
        // reconstruct spot dir from x/z and y-direction flag
        spot_dir.y = (1.0_f32 - spot_dir.x * spot_dir.x - spot_dir.z * spot_dir.z).sqrt();
        if (light.flags & POINT_LIGHT_FLAGS_SPOT_LIGHT_Y_NEGATIVE) != 0 {
            spot_dir.y = -spot_dir.y;
        }

        // view matrix z_axis is the reverse of transform.forward()
        let fwd = -spot_dir;
        let distance_to_light = fwd.dot(surface_to_light);
        let offset_position = -surface_to_light
            + (light.shadow_depth_bias * surface_to_light.normalize())
            + (surface_normal * light.shadow_normal_bias) * distance_to_light;

        // the construction of the up and right vectors needs to precisely mirror the code
        // in render/light.rs:spot_light_view_matrix
        let mut sign = -1.0;
        if fwd.z >= 0.0 {
            sign = 1.0;
        }
        let a = -1.0 / (fwd.z + sign);
        let b = fwd.x * fwd.y * a;
        let up_dir = Vec3::new(1.0 + sign * fwd.x * fwd.x * a, sign * b, -sign * fwd.x);
        let right_dir = Vec3::new(-b, -sign - fwd.y * fwd.y * a, fwd.y);
        let light_inv_rot = Mat3 {
            x_axis: right_dir,
            y_axis: up_dir,
            z_axis: fwd,
        };

        // because the matrix is a pure rotation matrix, the inverse is just the transpose, and to calculate
        // the product of the transpose with a vector we can just post-multiply instead of pre-multplying.
        // this allows us to keep the matrix construction code identical between CPU and GPU.
        let projected_position = light_inv_rot * offset_position;

        // divide xy by perspective matrix "f" and by -projected.z (projected.z is -projection matrix's w)
        // to get ndc coordinates
        let f_div_minus_z = 1.0 / (light.spot_light_tan_angle * -projected_position.z);
        let shadow_xy_ndc = projected_position.truncate() * f_div_minus_z;
        // convert to uv coordinates
        let shadow_uv = shadow_xy_ndc * Vec2::new(0.5, -0.5) + Vec2::new(0.5, 0.5);

        // 0.1 must match POINT_LIGHT_NEAR_Z
        let depth = 0.1 / -projected_position.z;

        #[cfg(feature = "NO_ARRAY_TEXTURES_SUPPORT")]
        {
            textureSampleCompare(
                directional_shadow_textures,
                directional_shadow_textures_sampler,
                shadow_uv,
                depth,
            )
        }

        #[cfg(not(feature = "NO_ARRAY_TEXTURES_SUPPORT"))]
        {
            directional_shadow_textures.sample_depth_reference_by_lod(
                *directional_shadow_textures_sampler,
                shadow_uv.extend(0.0),
                depth,
                light_id as f32 + lights.spot_light_shadowmap_offset as f32,
            )
        }
    }
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
