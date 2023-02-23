use spirv_std::{spirv, Image, Sampler};

use crate::prelude::StandardMaterial;

pub type BaseColorTexture = Image!(2D, type = f32);
pub type EmissiveTexture = Image!(2D, type = f32);
pub type MetallicRoughnessTexture = Image!(2D, type = f32);
pub type OcclusionTexture = Image!(2D, type = f32);
pub type NormalMapTexture = Image!(2D, type = f32);

#[allow(unused_variables)]
#[spirv(fragment)]
pub fn pbr_bindings(
    #[spirv(uniform, descriptor_set = 1, binding = 0)] material: &StandardMaterial,
    #[spirv(descriptor_set = 1, binding = 1)] base_color_texture: &BaseColorTexture,
    #[spirv(descriptor_set = 1, binding = 2)] base_color_sampler: &Sampler,
    #[spirv(descriptor_set = 1, binding = 3)] emissive_texture: &EmissiveTexture,
    #[spirv(descriptor_set = 1, binding = 4)] emissive_sampler: &Sampler,
    #[spirv(descriptor_set = 1, binding = 5)] metallic_roughness_texture: &MetallicRoughnessTexture,
    #[spirv(descriptor_set = 1, binding = 6)] metallic_roughness_sampler: &Sampler,
    #[spirv(descriptor_set = 1, binding = 7)] occlusion_texture: &OcclusionTexture,
    #[spirv(descriptor_set = 1, binding = 8)] occlusion_sampler: &Sampler,
    #[spirv(descriptor_set = 1, binding = 9)] normal_map_texture: &NormalMapTexture,
    #[spirv(descriptor_set = 1, binding = 10)] normal_map_sampler: &Sampler,
) {
}
