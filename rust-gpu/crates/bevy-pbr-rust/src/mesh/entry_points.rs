use spirv_std::{
    glam::{Vec2, Vec3, Vec4},
    spirv,
};

use permutate_macro::permutate;

use crate::prelude::{Mesh, Skinning, VertexNormal, VertexPosition, VertexTangent, View};

#[spirv(vertex)]
#[allow(non_snake_case)]
#[permutate(
    parameters = {
        tangent: some | none,
        color: some | none,
        skinned: some | none
    },
    permutations = [
        file("../../entry_points.json", "mesh::entry_points")
    ]
)]
pub fn vertex(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] view: &View,
    #[spirv(uniform, descriptor_set = 2, binding = 0)] mesh: &Mesh,

    #[permutate(skinned = some)]
    #[spirv(uniform, descriptor_set = 2, binding = 1)]
    joint_matrices: &crate::prelude::SkinnedMesh,

    in_position: Vec3,
    in_normal: Vec3,
    in_uv: Vec2,

    #[permutate(tangent = some)] in_tangent: Vec4,

    #[permutate(color = some)] in_color: Vec4,

    #[permutate(skinned = some)] in_joint_indices: rust_gpu_util::glam::UVec4,
    #[permutate(skinned = some)] in_joint_weights: rust_gpu_util::glam::Vec4,

    #[spirv(position)] out_clip_position: &mut Vec4,
    out_world_position: &mut Vec4,
    out_world_normal: &mut Vec3,
    out_uv: &mut Vec2,
    #[permutate(tangent = some)] out_tangent: &mut Vec4,
    #[permutate(color = some)] out_color: &mut Vec4,
) {
    let mut in_position = in_position.extend(1.0);
    let mut in_normal = in_normal;

    #[permutate(tangent = some)]
    let mut in_tangent = in_tangent;

    vertex_impl(
        view,
        mesh,
        &mut in_position,
        &mut in_normal,
        #[permutate(tangent = some)]
        &mut in_tangent,
        #[permutate(tangent = none)]
        &mut (),
        #[permutate(skinned = some)]
        joint_matrices,
        #[permutate(skinned = none)]
        &(),
        #[permutate(skinned = some)]
        in_joint_indices,
        #[permutate(skinned = none)]
        (),
        #[permutate(skinned = some)]
        in_joint_weights,
        #[permutate(skinned = none)]
        (),
        out_clip_position,
    );

    *out_world_position = in_position;
    *out_world_normal = in_normal;
    *out_uv = in_uv;

    #[permutate(tangent = some)]
    *out_tangent = in_tangent;

    #[permutate(color = some)]
    *out_color = in_color;
}

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

#[spirv(fragment)]
#[allow(unused_variables)]
pub fn fragment(
    #[spirv(position)] in_clip_position: Vec4,
    in_world_position: Vec4,
    in_world_normal: Vec3,
    in_uv: Vec2,
    out_color: &mut Vec4,
) {
    *out_color = Vec4::new(1.0, 0.0, 1.0, 1.0);
}
