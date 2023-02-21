pub mod mesh_bindings;
pub mod mesh_functions;
pub mod mesh_types;

use super::prelude::{mesh_position_local_to_world, Mesh, View};

use spirv_std::{
    glam::{Vec2, Vec3, Vec4},
    spirv,
};

#[allow(unused_imports)]
use spirv_std::num_traits::Float;

#[allow(unused_variables)]
#[spirv(vertex)]
pub fn vertex(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] view: &View,
    #[spirv(uniform, descriptor_set = 2, binding = 0)] mesh: &Mesh,
    #[cfg(feature = "vertex_positions")] in_position: Vec3,
    #[cfg(feature = "vertex_normals")] in_normal: Vec3,
    #[cfg(feature = "vertex_uvs")] in_uv: Vec2,
    #[cfg(feature = "vertex_tangents")] in_tangent: Vec4,
    #[cfg(feature = "vertex_colors")] in_color: Vec4,
    #[cfg(feature = "skinned")] in_joint_indices: Vec4,
    #[cfg(feature = "skinned")] in_joint_weights: Vec4,
    #[spirv(position)] out_clip_position: &mut Vec4,
    out_world_position: &mut Vec4,
    out_world_normal: &mut Vec3,
    #[cfg(feature = "vertex_uvs")] out_uv: &mut Vec2,
    #[cfg(feature = "vertex_tangents")] out_tangent: &mut Vec2,
    #[cfg(feature = "vertex_colors")] out_color: &mut Vec2,
) {
    #[cfg(feature = "skinned")]
    let mut model = skin_model(vertex.joint_indices, vertex.joint_weights);

    #[cfg(not(feature = "skinned"))]
    let model = mesh.model;

    #[cfg(feature = "skinned")]
    {
        out_world_normal = skin_normals(model, vertex.normal);
    }

    #[cfg(all(feature = "vertex_normals", not(feature = "skinned")))]
    {
        *out_world_normal = mesh.mesh_normal_local_to_world(in_normal);
    }

    #[cfg(feature = "vertex_positions")]
    {
        *out_world_position = mesh_position_local_to_world(model, in_position.extend(1.0));
        *out_clip_position = view.mesh_position_world_to_clip(*out_world_position);
    }

    #[cfg(feature = "vertex_uvs")]
    {
        *out_uv = in_uv;
    }

    #[cfg(feature = "vertex_tangents")]
    {
        out.world_tangent = mesh_tangent_local_to_world(model, vertex.tangent);
    }

    #[cfg(feature = "vertex_colors")]
    {
        out.color = vertex.color;
    }
}

#[allow(unused_variables)]
#[spirv(fragment)]
pub fn fragment(
    #[spirv(position)] in_clip_position: Vec4,
    in_world_position: Vec4,
    in_world_normal: Vec3,
    #[cfg(feature = "vertex_uvs")] in_uv: Vec2,
    #[cfg(feature = "vertex_tangents")] in_tangent: Vec2,
    #[cfg(feature = "vertex_colors")] in_color: Vec4,
    out_color: &mut Vec4,
) {
    *out_color = in_clip_position + in_world_position + in_world_normal.extend(0.0);

    #[cfg(feature = "vertex_uvs")]
    {
        *out_color = *out_color + in_uv.extend(0.0).extend(0.0);
    }

    #[cfg(feature = "vertex_colors")]
    {
        *out_color = input.color;
    }

    #[cfg(not(feature = "vertex_colors"))]
    {
        *out_color = Vec4::new(1.0, 0.0, 1.0, 1.0);
    }
}
