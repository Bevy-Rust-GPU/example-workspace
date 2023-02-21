use spirv_std::glam::{Mat3, Mat4, Vec3, Vec4};

use super::{
    mesh_types::{Mesh, MESH_FLAGS_SIGN_DETERMINANT_MODEL_3X3_BIT},
    mesh_view_types::View,
};

pub fn mesh_position_local_to_world(model: Mat4, vertex_position: Vec4) -> Vec4 {
    model * vertex_position
}

pub fn mesh_position_world_to_clip(view: &View, world_position: Vec4) -> Vec4 {
    view.view_proj * world_position
}

// NOTE: The intermediate world_position assignment is important
// for precision purposes when using the 'equals' depth comparison
// function.
pub fn mesh_position_local_to_clip(view: &View, model: Mat4, vertex_position: Vec4) -> Vec4 {
    let world_position = mesh_position_local_to_world(model, vertex_position);
    mesh_position_world_to_clip(view, world_position)
}

pub fn mesh_normal_local_to_world(mesh: &Mesh, vertex_normal: Vec3) -> Vec3 {
    // NOTE: The mikktspace method of normal mapping requires that the world normal is
    // re-normalized in the vertex shader to match the way mikktspace bakes vertex tangents
    // and normal maps so that the exact inverse process is applied when shading. Blender, Unity,
    // Unreal Engine, Godot, and more all use the mikktspace method. Do not change this code
    // unless you really know what you are doing.
    // http://www.mikktspace.com/
    (Mat3 {
        x_axis: mesh.inverse_transpose_model.x_axis.truncate(),
        y_axis: mesh.inverse_transpose_model.y_axis.truncate(),
        z_axis: mesh.inverse_transpose_model.z_axis.truncate(),
    } * vertex_normal)
        .normalize()
}

// Calculates the sign of the determinant of the 3x3 model matrix based on a
// mesh flag
pub fn sign_determinant_model_3x3(mesh: &Mesh) -> f32 {
    // bool(u32) is false if 0u else true
    // f32(bool) is 1.0 if true else 0.0
    // * 2.0 - 1.0 remaps 0.0 or 1.0 to -1.0 or 1.0 respectively
    (if mesh.flags & MESH_FLAGS_SIGN_DETERMINANT_MODEL_3X3_BIT != 0 {
        1.0
    } else {
        2.0
    } * 2.0
        - 1.0)
}

pub fn mesh_tangent_local_to_world(mesh: &Mesh, model: Mat4, vertex_tangent: Vec4) -> Vec4 {
    // NOTE: The mikktspace method of normal mapping requires that the world tangent is
    // re-normalized in the vertex shader to match the way mikktspace bakes vertex tangents
    // and normal maps so that the exact inverse process is applied when shading. Blender, Unity,
    // Unreal Engine, Godot, and more all use the mikktspace method. Do not change this code
    // unless you really know what you are doing.
    // http://www.mikktspace.com/
    (Mat3 {
        x_axis: model.x_axis.truncate(),
        y_axis: model.y_axis.truncate(),
        z_axis: model.z_axis.truncate(),
    } * vertex_tangent.truncate())
    .normalize()
    .extend(
        // NOTE: Multiplying by the sign of the determinant of the 3x3 model matrix accounts for
        // situations such as negative scaling.
        vertex_tangent.w * sign_determinant_model_3x3(mesh),
    )
}
