use spirv_std::{spirv, Image, Sampler};

use super::super::prelude::{
    ClusterLightIndexLists, ClusterOffsetsAndCounts, DirectionalShadowTextures, Globals, Lights,
    PointLights, PointShadowTextures, View,
};

pub type TextureDepthCube = Image!(cube, type = f32, sampled = true, depth = true);
pub type TextureDepthCubeArray =
    Image!(cube, type = f32, sampled = true, depth = true, arrayed = true);

pub type TextureDepth2d = Image!(2D, type = f32, sampled = true, depth = true);
pub type TextureDepth2dArray = Image!(2D, type = f32, sampled = true, depth = true, arrayed = true);

#[allow(unused_variables)]
#[spirv(fragment)]
pub fn mesh_view_bindings(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] view: &View,
    #[spirv(uniform, descriptor_set = 0, binding = 1)] lights: &Lights,

    #[spirv(descriptor_set = 0, binding = 2)] point_shadow_textures: &PointShadowTextures,

    #[spirv(descriptor_set = 0, binding = 3)] point_shadow_textures_sampler: &Sampler,

    #[spirv(descriptor_set = 0, binding = 4)]
    directional_shadow_textures: &DirectionalShadowTextures,

    #[spirv(descriptor_set = 0, binding = 5)] directional_shadow_textures_sampler: &Sampler,

    #[cfg(feature = "NO_STORAGE_BUFFERS_SUPPORT")]
    #[spirv(uniform, descriptor_set = 0, binding = 6)]
    point_lights: &PointLights,

    #[cfg(not(feature = "NO_STORAGE_BUFFERS_SUPPORT"))]
    #[spirv(storage_buffer, descriptor_set = 0, binding = 6)]
    point_lights: &PointLights,

    #[cfg(feature = "NO_STORAGE_BUFFERS_SUPPORT")]
    #[spirv(uniform, descriptor_set = 0, binding = 7)]
    cluster_light_index_lists: &ClusterLightIndexLists,

    #[cfg(not(feature = "NO_STORAGE_BUFFERS_SUPPORT"))]
    #[spirv(storage_buffer, descriptor_set = 0, binding = 7)]
    cluster_light_index_lists: &ClusterLightIndexLists,

    #[cfg(feature = "NO_STORAGE_BUFFERS_SUPPORT")]
    #[spirv(uniform, descriptor_set = 0, binding = 8)]
    cluster_offsets_and_counts: &ClusterOffsetsAndCounts,

    #[cfg(not(feature = "NO_STORAGE_BUFFERS_SUPPORT"))]
    #[spirv(storage_buffer, descriptor_set = 0, binding = 8)]
    cluster_offsets_and_counts: &ClusterOffsetsAndCounts,

    #[spirv(uniform, descriptor_set = 0, binding = 9)] globals: &Globals,
) {
}
