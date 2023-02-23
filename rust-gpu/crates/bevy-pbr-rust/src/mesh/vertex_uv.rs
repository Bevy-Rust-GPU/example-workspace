use crate::prelude::{
    BaseColorTexture, BaseMaterial, EmissiveTexture, MetallicRoughnessTexture, NormalMapTexture,
    OcclusionTexture, STANDARD_MATERIAL_FLAGS_BASE_COLOR_TEXTURE_BIT,
    STANDARD_MATERIAL_FLAGS_EMISSIVE_TEXTURE_BIT,
    STANDARD_MATERIAL_FLAGS_METALLIC_ROUGHNESS_TEXTURE_BIT,
    STANDARD_MATERIAL_FLAGS_OCCLUSION_TEXTURE_BIT, BaseMaterialNormalMap, VertexTangent,
};

use spirv_std::{
    glam::{Vec2, Vec3, Vec4},
    Sampler,
};

#[allow(unused_imports)]
use spirv_std::num_traits::Float;

pub trait VertexUv {
    fn sample_base_color_texture(
        &self,
        _base_color_texture: &BaseColorTexture,
        _sampler: &Sampler,
        _material: &BaseMaterial,
        _output_color: &mut Vec4,
    ) {
    }

    fn sample_emissive_texture(
        &self,
        _emissive_texture: &EmissiveTexture,
        _sampler: &Sampler,
        _material: &BaseMaterial,
        _emissive: &mut Vec4,
    ) {
    }

    fn sample_metallic_roughness_texture(
        &self,
        _metallic_roughness_texture: &MetallicRoughnessTexture,
        _sampler: &Sampler,
        _material: &BaseMaterial,
        _metallic: &mut f32,
        _perceptual_roughness: &mut f32,
    ) {
    }

    fn sample_occlusion_texture(
        &self,
        _occlusion_texture: &OcclusionTexture,
        _sampler: &Sampler,
        _material: &BaseMaterial,
        _occlusion: &mut f32,
    ) {
    }

    fn apply_pbr_input_n<VT: VertexTangent, N: BaseMaterialNormalMap>(
        &self,
        _vt: &VT,
        _standard_material_flags: u32,
        _normal_map_texture: &NormalMapTexture,
        _normal_map_sampler: &Sampler,
        _n: &mut Vec3,
    ) {
    }
}

impl VertexUv for Vec2 {
    fn sample_base_color_texture(
        &self,
        base_color_texture: &BaseColorTexture,
        sampler: &Sampler,
        material: &BaseMaterial,
        output_color: &mut Vec4,
    ) {
        if (material.base.flags & STANDARD_MATERIAL_FLAGS_BASE_COLOR_TEXTURE_BIT) != 0 {
            *output_color = *output_color * base_color_texture.sample::<f32, Vec4>(*sampler, *self);
        }
    }

    fn sample_emissive_texture(
        &self,
        emissive_texture: &EmissiveTexture,
        sampler: &Sampler,
        material: &BaseMaterial,
        emissive: &mut Vec4,
    ) {
        if (material.base.flags & STANDARD_MATERIAL_FLAGS_EMISSIVE_TEXTURE_BIT) != 0 {
            *emissive = (emissive.truncate()
                * emissive_texture
                    .sample::<f32, Vec4>(*sampler, *self)
                    .truncate())
            .extend(1.0);
        }
    }

    fn sample_metallic_roughness_texture(
        &self,
        metallic_roughness_texture: &MetallicRoughnessTexture,
        sampler: &Sampler,
        material: &BaseMaterial,
        metallic: &mut f32,
        perceptual_roughness: &mut f32,
    ) {
        if (material.base.flags & STANDARD_MATERIAL_FLAGS_METALLIC_ROUGHNESS_TEXTURE_BIT) != 0 {
            let metallic_roughness =
                metallic_roughness_texture.sample::<f32, Vec4>(*sampler, *self);
            // Sampling from GLTF standard channels for now
            *metallic = *metallic * metallic_roughness.z;
            *perceptual_roughness = *perceptual_roughness * metallic_roughness.y;
        }
    }

    fn sample_occlusion_texture(
        &self,
        occlusion_texture: &OcclusionTexture,
        sampler: &Sampler,
        material: &BaseMaterial,
        occlusion: &mut f32,
    ) {
        if (material.base.flags & STANDARD_MATERIAL_FLAGS_OCCLUSION_TEXTURE_BIT) != 0 {
            *occlusion = occlusion_texture.sample::<f32, Vec4>(*sampler, *self).x;
        }
    }

    fn apply_pbr_input_n<VT: VertexTangent, N: BaseMaterialNormalMap>(
        &self,
        vt: &VT,
        standard_material_flags: u32,
        normal_map_texture: &NormalMapTexture,
        normal_map_sampler: &Sampler,
        n: &mut Vec3,
    ) {
        vt.apply_pbr_input_n::<N>(
            n,
            *self,
            standard_material_flags,
            normal_map_texture,
            normal_map_sampler,
        );
    }
}

impl VertexUv for () {}
