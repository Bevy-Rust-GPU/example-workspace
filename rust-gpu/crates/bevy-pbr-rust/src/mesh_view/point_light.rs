use spirv_std::glam::{Vec3, Vec4};

#[allow(unused_imports)]
use spirv_std::num_traits::Float;

use rust_gpu_util::saturate::Saturate;

use super::super::prelude::{fd_burley, get_distance_attenuation, specular};

pub const POINT_LIGHT_FLAGS_SHADOWS_ENABLED_BIT: u32 = 1;
pub const POINT_LIGHT_FLAGS_SPOT_LIGHT_Y_NEGATIVE: u32 = 2;

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
