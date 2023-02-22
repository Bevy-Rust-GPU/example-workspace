use crate::prelude::{BaseMaterialNormalMap, Mesh, NormalMapTexture};

use spirv_std::{
    glam::{Mat4, Vec2, Vec3, Vec4},
    Sampler,
};

#[allow(unused_imports)]
use spirv_std::num_traits::Float;
pub trait VertexTangent {
    const PRESENT: bool = false;

    fn transform_tangent(&mut self, _mesh: &Mesh, _model: Mat4) {}
    fn apply_flip_factor<N: BaseMaterialNormalMap>(
        _double_sided: bool,
        _is_front: bool,
        _normal: &mut Vec3,
    ) {
    }
    fn apply_pbr_input_n<N: BaseMaterialNormalMap>(
        &self,
        _n: &mut Vec3,
        _world_uv: Vec2,
        _standard_material_flags: u32,
        _normal_map_texture: &NormalMapTexture,
        _normal_map_sampler: &Sampler,
    ) {
    }
}

impl VertexTangent for Vec4 {
    const PRESENT: bool = true;

    fn transform_tangent(&mut self, mesh: &Mesh, model: Mat4) {
        *self = mesh.mesh_tangent_local_to_world(model, *self);
    }

    fn apply_flip_factor<N: BaseMaterialNormalMap>(
        double_sided: bool,
        is_front: bool,
        normal: &mut Vec3,
    ) {
        N::apply_flip_factor(double_sided, is_front, normal);
    }

    fn apply_pbr_input_n<N: BaseMaterialNormalMap>(
        &self,
        n: &mut Vec3,
        uv: Vec2,
        standard_material_flags: u32,
        normal_map_texture: &NormalMapTexture,
        normal_map_sampler: &Sampler,
    ) {
        N::apply_pbr_input_n(
            uv,
            *self,
            standard_material_flags,
            normal_map_texture,
            normal_map_sampler,
            n,
        );
    }
}

impl VertexTangent for () {}
