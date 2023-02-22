pub mod base_material_normal_map;
pub mod bindings;
pub mod entry_points;
pub mod skinned_mesh;
pub mod skinning;
pub mod vertex_color;
pub mod vertex_normal;
pub mod vertex_position;
pub mod vertex_tangent;
pub mod vertex_uv;

use spirv_std::glam::{Mat3, Mat4, Vec3, Vec4};

pub const MESH_FLAGS_SHADOW_RECEIVER_BIT: u32 = 1;
// 2^31 - if the flag is set, the sign is positive, else it is negative
pub const MESH_FLAGS_SIGN_DETERMINANT_MODEL_3X3_BIT: u32 = 2147483648;

pub fn mesh_position_local_to_world(model: Mat4, vertex_position: Vec4) -> Vec4 {
    model * vertex_position
}

#[repr(C)]
pub struct Mesh {
    pub model: Mat4,
    pub inverse_transpose_model: Mat4,
    // 'flags' is a bit field indicating various options. u32 is 32 bits so we have up to 32 options.
    pub flags: u32,
}

impl Mesh {
    pub fn mesh_normal_local_to_world(&self, vertex_normal: Vec3) -> Vec3 {
        // NOTE: The mikktspace method of normal mapping requires that the world normal is
        // re-normalized in the vertex shader to match the way mikktspace bakes vertex tangents
        // and normal maps so that the exact inverse process is applied when shading. Blender, Unity,
        // Unreal Engine, Godot, and more all use the mikktspace method. Do not change this code
        // unless you really know what you are doing.
        // http://www.mikktspace.com/
        (Mat3 {
            x_axis: self.inverse_transpose_model.x_axis.truncate(),
            y_axis: self.inverse_transpose_model.y_axis.truncate(),
            z_axis: self.inverse_transpose_model.z_axis.truncate(),
        } * vertex_normal)
            .normalize()
    }

    // Calculates the sign of the determinant of the 3x3 model matrix based on a
    // mesh flag
    pub fn sign_determinant_model_3x3(&self) -> f32 {
        // bool(u32) is false if 0u else true
        // f32(bool) is 1.0 if true else 0.0
        // * 2.0 - 1.0 remaps 0.0 or 1.0 to -1.0 or 1.0 respectively
        (if self.flags & MESH_FLAGS_SIGN_DETERMINANT_MODEL_3X3_BIT != 0 {
            1.0
        } else {
            2.0
        } * 2.0
            - 1.0)
    }

    pub fn mesh_tangent_local_to_world(&self, model: Mat4, vertex_tangent: Vec4) -> Vec4 {
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
            vertex_tangent.w * self.sign_determinant_model_3x3(),
        )
    }
}
