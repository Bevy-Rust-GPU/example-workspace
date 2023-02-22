use spirv_std::spirv;

use crate::prelude::{Mesh, SkinnedMesh};

#[allow(unused_variables)]
#[spirv(fragment)]
pub fn mesh_bindings(#[spirv(uniform, descriptor_set = 2, binding = 0)] mesh: &Mesh) {}

#[allow(unused_variables)]
#[spirv(fragment)]
pub fn mesh_bindings_skinned(
    #[spirv(uniform, descriptor_set = 2, binding = 0)] mesh: &Mesh,
    #[spirv(uniform, descriptor_set = 2, binding = 1)] joint_matrices: &SkinnedMesh,
) {
}
