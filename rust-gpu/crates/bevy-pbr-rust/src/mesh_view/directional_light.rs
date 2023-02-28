use spirv_std::glam::{Mat4, Vec3, Vec4};

use rust_gpu_util::saturate::Saturate;

use crate::prelude::{fd_burley, specular};

pub const DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT: u32 = 1;

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
