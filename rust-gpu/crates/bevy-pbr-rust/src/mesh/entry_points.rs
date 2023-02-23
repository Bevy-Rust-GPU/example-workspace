use spirv_std::{
    glam::{Vec2, Vec3, Vec4},
    spirv,
};

use crate::prelude::{Mesh, Skinning, VertexNormal, VertexPosition, VertexTangent, View};

pub fn vertex_impl<P: VertexPosition, N: VertexNormal, T: VertexTangent, SM: Skinning>(
    view: &View,
    mesh: &Mesh,
    vertex_position: &mut P,
    vertex_normal: &mut N,
    vertex_tangent: &mut T,
    joint_matrices: &SM,
    in_joint_indices: SM::JointIndices,
    in_joint_weights: SM::JointWeights,
    out_clip_position: &mut Vec4,
) {
    let model = joint_matrices.skin_model(mesh, in_joint_indices, in_joint_weights);

    vertex_normal.skin_normals::<SM>(mesh, model);
    vertex_position.transform_position(view, mesh, model, out_clip_position);
    vertex_tangent.transform_tangent(mesh, model);
}

#[spirv(vertex)]
#[allow(non_snake_case)]
pub fn vertex__position__normal__none__none(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] view: &View,
    #[spirv(uniform, descriptor_set = 2, binding = 0)] mesh: &Mesh,
    /* skinning
    #[spirv(uniform, descriptor_set = 2, binding = 1)]
    joint_matrices: &SkinnedMesh,
    */
    in_position: Vec3,
    in_normal: Vec3,
    in_uv: Vec2,
    /* in vertex attributes
        in_tangent: Vec4,
        in_color: Vec4,
        */
    /* skinning
        in_joint_indices: Vec4,
        in_joint_weights: Vec4,
        */
    #[spirv(position)] out_clip_position: &mut Vec4,
    out_world_position: &mut Vec4,
    out_world_normal: &mut Vec3,
    out_uv: &mut Vec2,
    /* out vertex attributes
    out_tangent: &mut Vec2,
    out_color: &mut Vec2,
    */
) {
    let mut in_position = in_position.extend(1.0);
    let mut in_normal = in_normal;

    vertex_impl(
        view,
        mesh,
        &mut in_position,
        &mut in_normal,
        &mut (), //in_tangent,
        &(),     //joint_matrices,
        (),      //in_joint_indices,
        (),      //in_joint_weights,
        out_clip_position,
    );

    *out_world_position = in_position;
    *out_world_normal = in_normal;
    *out_uv = in_uv;
}

#[allow(unused_variables)]
#[spirv(fragment)]
pub fn fragment(
    #[spirv(position)] in_clip_position: Vec4,
    in_world_position: Vec4,
    in_world_normal: Vec3,
    in_uv: Vec2,
    out_color: &mut Vec4,
) {
    *out_color = Vec4::new(1.0, 0.0, 1.0, 1.0);
}
