#![no_std]

pub use bevy_pbr_rust;

use bevy_pbr_rust::prelude::{Globals, Mesh, View};
use spirv_std::{
    glam::{Vec3, Vec4},
    spirv,
};

#[allow(unused_imports)]
use spirv_std::num_traits::Float;

#[spirv(vertex)]
pub fn vertex_warp(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] view: &View,
    #[spirv(uniform, descriptor_set = 2, binding = 0)] mesh: &Mesh,
    #[spirv(uniform, descriptor_set = 0, binding = 9)] globals: &Globals,

    in_position: Vec3,
    in_normal: Vec3,

    #[spirv(position)] out_clip_position: &mut Vec4,
    out_world_normal: &mut Vec3,
) {
    let mut position_local = in_position.extend(1.0);

    position_local.x += position_local.x * position_local.z * globals.time.sin();
    position_local.y += position_local.y * position_local.z * globals.time.cos();
    position_local.z += position_local.z * globals.time.sin() * globals.time.cos();

    let position_world = mesh.model * position_local;
    let position_clip = view.view_proj * position_world;

    *out_clip_position = position_clip;
    *out_world_normal = in_normal;
}

#[spirv(fragment)]
#[allow(unused_variables)]
pub fn fragment_normal(
    #[spirv(position)] in_clip_position: Vec4,
    in_world_normal: Vec3,
    out_color: &mut Vec4,
) {
    *out_color = in_world_normal.extend(1.0);
}
