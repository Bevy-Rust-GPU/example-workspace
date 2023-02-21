use spirv_std::glam::Vec4;
use spirv_std::glam::{Vec2, Vec3};
use spirv_std::macros::spirv;
use spirv_std::Sampler;

use crate::bevy_core_pipeline::tonemapping_shared::screen_space_dither;
use crate::BaseMaterial;

use super::mesh_types::Mesh;
use super::mesh_view_types::{
    ClusterLightIndexLists, ClusterOffsetsAndCounts, Lights, PointLights, View,
};
use super::pbr_bindings::{
    BaseColorTexture, EmissiveTexture, MetallicRoughnessTexture, OcclusionTexture,
};
use super::pbr_functions::{
    alpha_discard, apply_normal_mapping, calculate_view, pbr, prepare_world_normal, tone_mapping,
    PbrInput,
};
use super::pbr_types::{
    StandardMaterial, STANDARD_MATERIAL_FLAGS_BASE_COLOR_TEXTURE_BIT,
    STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT, STANDARD_MATERIAL_FLAGS_EMISSIVE_TEXTURE_BIT,
    STANDARD_MATERIAL_FLAGS_METALLIC_ROUGHNESS_TEXTURE_BIT,
    STANDARD_MATERIAL_FLAGS_OCCLUSION_TEXTURE_BIT, STANDARD_MATERIAL_FLAGS_UNLIT_BIT,
};
use super::shadows::{DirectionalShadowTextures, PointShadowTextures};

#[spirv(fragment)]
pub fn fragment(
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

    #[spirv(uniform, descriptor_set = 1, binding = 0)] material: &BaseMaterial,
    #[spirv(descriptor_set = 1, binding = 1)] base_color_texture: &BaseColorTexture,
    #[spirv(descriptor_set = 1, binding = 2)] base_color_sampler: &Sampler,
    #[spirv(descriptor_set = 1, binding = 3)] emissive_texture: &EmissiveTexture,
    #[spirv(descriptor_set = 1, binding = 4)] emissive_sampler: &Sampler,
    #[spirv(descriptor_set = 1, binding = 5)] metallic_roughness_texture: &MetallicRoughnessTexture,
    #[spirv(descriptor_set = 1, binding = 6)] metallic_roughness_sampler: &Sampler,
    #[spirv(descriptor_set = 1, binding = 7)] occlusion_texture: &OcclusionTexture,
    #[spirv(descriptor_set = 1, binding = 8)] occlusion_sampler: &Sampler,

    #[spirv(uniform, descriptor_set = 2, binding = 0)] mesh: &Mesh,

    #[cfg(feature = "SKINNED")]
    #[spirv(uniform, descriptor_set = 2 binding = 1)]
    joint_matrices: SkinnedMesh,

    #[spirv(front_facing)] in_is_front: bool,
    #[spirv(position)] in_frag_coord: Vec4,
    in_world_position: Vec4,
    in_world_normal: Vec3,
    #[cfg(feature = "VERTEX_UVS")] in_uv: Vec2,
    #[cfg(feature = "VERTEX_TANGENTS")] in_tangent: Vec2,
    #[cfg(feature = "VERTEX_COLORS")] in_color: Vec2,

    output_color: &mut Vec4,
) {
    *output_color = material.base.base_color;

    #[cfg(feature = "VERTEX_COLORS")]
    {
        *output_color = *output_color * in_color;
    }

    #[cfg(feature = "VERTEX_UVS")]
    {
        if (material.base.flags & STANDARD_MATERIAL_FLAGS_BASE_COLOR_TEXTURE_BIT) != 0 {
            *output_color =
                *output_color * base_color_texture.sample::<f32, Vec4>(*base_color_sampler, in_uv);
        }
    }

    // NOTE: Unlit bit not set means == 0 is true, so the true case is if lit
    if material.base.flags & STANDARD_MATERIAL_FLAGS_UNLIT_BIT == 0 {
        // Prepare a 'processed' StandardMaterial by sampling all textures to resolve
        // the material members
        let mut pbr_input = PbrInput::default();

        pbr_input.material.base_color = *output_color;
        pbr_input.material.reflectance = material.base.reflectance;
        pbr_input.material.flags = material.base.flags;
        pbr_input.material.alpha_cutoff = material.base.alpha_cutoff;

        // TODO use .a for exposure compensation in HDR
        let mut emissive = material.base.emissive;

        #[cfg(feature = "VERTEX_UVS")]
        {
            if (material.base.flags & STANDARD_MATERIAL_FLAGS_EMISSIVE_TEXTURE_BIT) != 0 {
                emissive = (emissive.truncate()
                    * emissive_texture
                        .sample::<f32, Vec4>(*emissive_sampler, in_uv)
                        .truncate())
                .extend(1.0);
            }
        }

        pbr_input.material.emissive = emissive;

        let mut metallic = material.base.metallic;
        let mut perceptual_roughness = material.base.perceptual_roughness;

        #[cfg(feature = "VERTEX_UVS")]
        {
            if (material.base.flags & STANDARD_MATERIAL_FLAGS_METALLIC_ROUGHNESS_TEXTURE_BIT) != 0 {
                let metallic_roughness = metallic_roughness_texture
                    .sample::<f32, Vec4>(*metallic_roughness_sampler, in_uv);
                // Sampling from GLTF standard channels for now
                metallic = metallic * metallic_roughness.z;
                perceptual_roughness = perceptual_roughness * metallic_roughness.y;
            }
        }

        pbr_input.material.metallic = metallic;
        pbr_input.material.perceptual_roughness = perceptual_roughness;

        let mut occlusion: f32 = 1.0;

        #[cfg(feature = "VERTEX_UVS")]
        {
            if (material.base.flags & STANDARD_MATERIAL_FLAGS_OCCLUSION_TEXTURE_BIT) != 0 {
                occlusion = occlusion_texture
                    .sample::<f32, Vec4>(*occlusion_sampler, in_uv)
                    .x;
            }
        }

        pbr_input.occlusion = occlusion;

        pbr_input.frag_coord = in_frag_coord;
        pbr_input.world_position = in_world_position;
        pbr_input.world_normal = prepare_world_normal(
            in_world_normal,
            (material.base.flags & STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0,
            in_is_front,
        );

        pbr_input.is_orthographic = view.projection.w_axis.w == 1.0;

        pbr_input.n = apply_normal_mapping(
            material.base.flags,
            pbr_input.world_normal,
            #[cfg(all(feature = "VERTEX_TANGENTS", feature = "STANDARDMATERIAL_NORMAL_MAP"))]
            in_world_tangent,
            #[cfg(feature = "VERTEX_UVS")]
            in_uv,
        );
        pbr_input.v = calculate_view(view, in_world_position, pbr_input.is_orthographic);

        *output_color = pbr(
            view,
            mesh,
            lights,
            point_lights,
            cluster_light_index_lists,
            &cluster_offsets_and_counts,
            directional_shadow_textures,
            directional_shadow_textures_sampler,
            point_shadow_textures,
            point_shadow_textures_sampler,
            pbr_input,
        );
    } else {
        *output_color = alpha_discard(&material.base, *output_color);
    }

    #[cfg(feature = "TONEMAP_IN_SHADER")]
    {
        *output_color = tone_mapping(*output_color);
    }

    #[cfg(feature = "DEBAND_DITHER")]
    {
        let mut output_rgb = output_color.truncate();
        output_rgb = output_rgb.powf(1.0 / 2.2);
        output_rgb = output_rgb + screen_space_dither(in_frag_coord.truncate().truncate());
        // This conversion back to linear space is required because our output texture format is
        // SRGB; the GPU will assume our output is linear and will apply an SRGB conversion.
        output_rgb = output_rgb.powf(2.2);
        *output_color = output_rgb.extend(output_color.w);
    }
}
