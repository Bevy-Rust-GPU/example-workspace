use spirv_std::glam::{Mat4, Vec4};

pub fn mesh_position_local_to_world(model: Mat4, vertex_position: Vec4) -> Vec4 {
    model * vertex_position
}
