use spirv_std::spirv;

use super::mesh_types::Mesh;

#[allow(unused_variables)]
#[spirv(fragment)]
pub fn mesh_bindings(
    #[spirv(uniform, descriptor_set = 2, binding = 0)] mesh: &Mesh,

    #[cfg(feature = "SKINNED")]
    #[spirv(uniform, descriptor_set = 2 binding = 1)]
    joint_matrices: SkinnedMesh,
) {
}

