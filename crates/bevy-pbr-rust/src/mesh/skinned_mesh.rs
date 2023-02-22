use spirv_std::glam::{Mat4, UVec4, Vec3, Vec4};

use crate::skinning::skin_normals;

use crate::prelude::Skinning;

use super::Mesh;

#[repr(C)]
pub struct SkinnedMesh {
    pub data: [Mat4; 256],
}

impl Skinning for SkinnedMesh {
    type JointIndices = UVec4;
    type JointWeights = Vec4;

    fn skin_model(&self, _: &Mesh, indexes: UVec4, weights: Vec4) -> Mat4 {
        weights.x * self.data[indexes.x as usize]
            + weights.y * self.data[indexes.y as usize]
            + weights.z * self.data[indexes.z as usize]
            + weights.w * self.data[indexes.w as usize]
    }

    fn skin_normals(_: &Mesh, model: Mat4, normal: Vec3) -> Vec3 {
        skin_normals(model, normal)
    }
}

