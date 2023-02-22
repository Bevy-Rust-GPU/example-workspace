use crate::prelude::{NormalMapTexture, PbrInput};

use crate::prelude::{BaseMaterialNormalMap, Mesh, Skinning, VertexTangent, VertexUv};

use spirv_std::{
    glam::{Mat4, Vec3},
    Sampler,
};

#[allow(unused_imports)]
use spirv_std::num_traits::Float;
pub trait VertexNormal {
    fn skin_normals<SM: Skinning>(&mut self, _mesh: &Mesh, _model: Mat4) {}

    fn prepare_world_normal<VT: VertexTangent, N: BaseMaterialNormalMap>(
        &self,
        _double_sided: bool,
        _is_front: bool,
        _pbr_input: &mut PbrInput,
    ) {
    }

    fn apply_pbr_input_n<VU: VertexUv, VT: VertexTangent, N: BaseMaterialNormalMap>(
        &self,
        _standard_material_flags: u32,
        _uv: &VU,
        _world_tangent: &VT,
        _normal_map_texture: &NormalMapTexture,
        _normal_map_sampler: &Sampler,
        _pbr_input: &mut PbrInput,
    ) {
    }
}

impl VertexNormal for Vec3 {
    fn skin_normals<SM: Skinning>(&mut self, mesh: &Mesh, model: Mat4) {
        *self = SM::skin_normals(mesh, model, *self);
    }

    fn prepare_world_normal<VT: VertexTangent, N: BaseMaterialNormalMap>(
        &self,
        double_sided: bool,
        is_front: bool,
        pbr_input: &mut PbrInput,
    ) {
        let mut output: Vec3 = *self;

        VT::apply_flip_factor::<N>(double_sided, is_front, &mut output);

        pbr_input.world_normal = output;
    }

    fn apply_pbr_input_n<VU: VertexUv, VT: VertexTangent, N: BaseMaterialNormalMap>(
        &self,
        standard_material_flags: u32,
        uv: &VU,
        world_tangent: &VT,
        normal_map_texture: &NormalMapTexture,
        normal_map_sampler: &Sampler,
        pbr_input: &mut PbrInput,
    ) {
        // NOTE: The mikktspace method of normal mapping explicitly requires that the world normal NOT
        // be re-normalized in the fragment shader. This is primarily to match the way mikktspace
        // bakes vertex tangents and normal maps so that this is the exact inverse. Blender, Unity,
        // Unreal Engine, Godot, and more all use the mikktspace method. Do not change this code
        // unless you really know what you are doing.
        // http://www.mikktspace.com/
        let mut n = *self;

        uv.apply_pbr_input_n::<VT, N>(
            world_tangent,
            standard_material_flags,
            normal_map_texture,
            normal_map_sampler,
            &mut n,
        );

        pbr_input.n = n.normalize();
    }
}

impl VertexNormal for () {}
