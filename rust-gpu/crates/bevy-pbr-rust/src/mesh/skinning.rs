use crate::prelude::Mesh;

use spirv_std::glam::{Mat4, Vec3};

#[allow(unused_imports)]
use spirv_std::num_traits::Float;
pub trait Skinning {
    type JointIndices;
    type JointWeights;

    fn skin_model(
        &self,
        mesh: &Mesh,
        indexes: Self::JointIndices,
        weights: Self::JointWeights,
    ) -> Mat4;

    fn skin_normals(mesh: &Mesh, model: Mat4, normal: Vec3) -> Vec3;
}

impl Skinning for () {
    type JointIndices = ();
    type JointWeights = ();

    fn skin_model(&self, mesh: &Mesh, _: Self::JointIndices, _: Self::JointWeights) -> Mat4 {
        mesh.model
    }

    fn skin_normals(mesh: &Mesh, _: Mat4, normal: Vec3) -> Vec3 {
        mesh.mesh_normal_local_to_world(normal)
    }
}
