use crate::prelude::{mesh_position_local_to_world, Mesh, View, PbrInput};

use spirv_std::glam::{Mat4, Vec4};

#[allow(unused_imports)]
use spirv_std::num_traits::Float;

pub trait VertexPosition {
    fn transform_position(
        &mut self,
        _view: &View,
        _mesh: &Mesh,
        _model: Mat4,
        _out_clip_position: &mut Vec4,
    ) {
    }

    fn apply_pbr_position(&self, _pbr_input: &mut PbrInput) {}
    fn apply_pbr_v(&self, _view: &View, _pbr_input: &mut PbrInput) {}
}

impl VertexPosition for Vec4 {
    fn transform_position(
        &mut self,
        view: &View,
        _: &Mesh,
        model: Mat4,
        out_clip_position: &mut Vec4,
    ) {
        *self = mesh_position_local_to_world(model, *self);
        *out_clip_position = view.mesh_position_world_to_clip(*self);
    }

    fn apply_pbr_position(&self, pbr_input: &mut PbrInput) {
        pbr_input.world_position = *self;
    }

    fn apply_pbr_v(&self, view: &View, pbr_input: &mut PbrInput) {
        pbr_input.v = view.calculate_view(*self, pbr_input.is_orthographic);
    }
}

impl VertexPosition for () {}
