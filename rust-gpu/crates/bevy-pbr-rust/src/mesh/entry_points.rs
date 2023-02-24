use shader_util::glam::UVec4;
use spirv_std::{
    glam::{Vec2, Vec3, Vec4},
    spirv,
};

use bevy_rust_gpu_macros::permutate;

use crate::prelude::{
    Mesh, SkinnedMesh, Skinning, VertexNormal, VertexPosition, VertexTangent, View,
};

#[permutate(
    mappings = {
        tangent: tangent | none,
        color: color | none,
        skinned: skinned | none
    },
    permutations = [
        (none, none, none),
        (tangent, none, none),
        (tangent, color, none),
        (tangent, none, skinned),
        (none, color, skinned),
        (none, color, none),
        (none, none, skinned),
        (tangent, color, skinned),
    ]
)]
#[allow(non_snake_case)]
#[spirv(vertex)]
pub fn vertex(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] view: &View,
    #[spirv(uniform, descriptor_set = 2, binding = 0)] mesh: &Mesh,

    #[permutate(skinned = skinned)]
    #[spirv(uniform, descriptor_set = 2, binding = 1)]
    joint_matrices: &SkinnedMesh,

    in_position: Vec3,
    in_normal: Vec3,
    in_uv: Vec2,

    #[permutate(tangent = tangent)] in_tangent: Vec4,

    #[permutate(color = color)] in_color: Vec4,

    #[permutate(skinned = skinned)] in_joint_indices: UVec4,
    #[permutate(skinned = skinned)] in_joint_weights: Vec4,

    #[spirv(position)] out_clip_position: &mut Vec4,
    out_world_position: &mut Vec4,
    out_world_normal: &mut Vec3,
    out_uv: &mut Vec2,
    #[permutate(tangent = tangent)] out_tangent: &mut Vec4,
    #[permutate(color = color)] out_color: &mut Vec4,
) {
    let mut in_position = in_position.extend(1.0);
    let mut in_normal = in_normal;

    #[permutate(tangent = tangent)]
    let mut in_tangent = in_tangent;

    vertex_impl(
        view,
        mesh,
        &mut in_position,
        &mut in_normal,
        #[permutate(tangent = tangent)]
        &mut in_tangent,
        #[permutate(tangent = none)]
        &mut (),
        #[permutate(skinned = skinned)]
        joint_matrices,
        #[permutate(skinned = none)]
        &(),
        #[permutate(skinned = skinned)]
        in_joint_indices,
        #[permutate(skinned = none)]
        (),
        #[permutate(skinned = skinned)]
        in_joint_weights,
        #[permutate(skinned = none)]
        (),
        out_clip_position,
    );

    *out_world_position = in_position;
    *out_world_normal = in_normal;
    *out_uv = in_uv;

    #[permutate(tangent = tangent)]
    *out_tangent = in_tangent;

    #[permutate(color = color)]
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
