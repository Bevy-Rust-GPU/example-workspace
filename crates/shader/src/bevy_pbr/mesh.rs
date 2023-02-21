use crate::bevy_pbr::{
    mesh_functions::{mesh_position_local_to_world, mesh_position_world_to_clip},
    mesh_types::Mesh,
    mesh_view_types::View,
};

use spirv_std::{
    glam::{Vec2, Vec3, Vec4},
    spirv,
};

#[allow(unused_imports)]
use spirv_std::num_traits::Float;

use super::mesh_functions::mesh_normal_local_to_world;

#[allow(unused_variables)]
#[spirv(vertex)]
pub fn vertex(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] view: &View,
    #[spirv(uniform, descriptor_set = 2, binding = 0)] mesh: &Mesh,
    #[cfg(feature = "VERTEX_POSITIONS")] in_position: Vec3,
    #[cfg(feature = "VERTEX_NORMALS")] in_normal: Vec3,
    #[cfg(feature = "VERTEX_UVS")] in_uv: Vec2,
    #[cfg(feature = "VERTEX_TANGENTS")] in_tangent: Vec4,
    #[cfg(feature = "VERTEX_COLORS")] in_color: Vec4,
    #[cfg(feature = "SKINNED")] in_joint_indices: Vec4,
    #[cfg(feature = "SKINNED")] in_joint_weights: Vec4,
    #[spirv(position)] out_clip_position: &mut Vec4,
    out_world_position: &mut Vec4,
    out_world_normal: &mut Vec3,
    #[cfg(feature = "VERTEX_UVS")] out_uv: &mut Vec2,
    #[cfg(feature = "VERTEX_TANGENTS")] out_tangent: &mut Vec2,
    #[cfg(feature = "VERTEX_COLORS")] out_color: &mut Vec2,
) {
    #[cfg(feature = "SKINNED")]
    let mut model = skin_model(vertex.joint_indices, vertex.joint_weights);

    #[cfg(not(feature = "SKINNED"))]
    let model = mesh.model;

    #[cfg(feature = "SKINNED")]
    {
        out_world_normal = skin_normals(model, vertex.normal);
    }

    #[cfg(not(feature = "SKINNED"))]
    {
        *out_world_normal = mesh_normal_local_to_world(mesh, in_normal);
    }

    #[cfg(feature = "VERTEX_POSITIONS")]
    {
        *out_world_position = mesh_position_local_to_world(model, in_position.extend(1.0));
        *out_clip_position = mesh_position_world_to_clip(view, *out_world_position);
    }

    #[cfg(feature = "VERTEX_UVS")]
    {
        *out_uv = in_uv;
    }

    #[cfg(feature = "VERTEX_TANGENTS")]
    {
        out.world_tangent = mesh_tangent_local_to_world(model, vertex.tangent);
    }

    #[cfg(feature = "VERTEX_COLORS")]
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
    #[cfg(feature = "VERTEX_UVS")] in_uv: Vec2,
    #[cfg(feature = "VERTEX_TANGENTS")] in_tangent: Vec2,
    #[cfg(feature = "VERTEX_COLORS")] in_color: Vec4,
    out_color: &mut Vec4,
) {
    *out_color = in_clip_position
        + in_world_position
        + in_world_normal.extend(0.0)
        + in_uv.extend(0.0).extend(0.0);

    #[cfg(feature = "VERTEX_COLORS")]
    {
        *out_color = input.color;
    }

    #[cfg(not(feature = "VERTEX_COLORS"))]
    {
        *out_color = Vec4::new(1.0, 0.0, 1.0, 1.0);
    }
}
