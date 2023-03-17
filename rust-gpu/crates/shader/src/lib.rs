#![no_std]

pub use bevy_pbr_rust;

use bevy_pbr_rust::{
    prelude::{Globals, Mesh, View},
    tonemapping_shared::screen_space_dither,
};
use rust_gpu_sdf::{
    default,
    prelude::{
        CentralDiffGradient, CentralDiffNormal, Circle, Cube, Octahedron, Point, Sphere,
        TetrahedronGradient, TetrahedronNormal, Torus,
    },
    raymarch::Raymarch,
    signed_distance_field::SignedDistanceField,
    type_fields::field::Field,
};
use spirv_std::{
    glam::{Mat3, Quat, Vec3, Vec4, Vec4Swizzles},
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

#[spirv(vertex)]
pub fn vertex_sdf(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] view: &View,
    #[spirv(uniform, descriptor_set = 2, binding = 0)] mesh: &Mesh,
    #[spirv(uniform, descriptor_set = 0, binding = 9)] globals: &Globals,

    in_position: Vec3,
    in_normal: Vec3,

    #[spirv(position)] out_clip_position: &mut Vec4,
    out_world_position: &mut Vec4,
    out_world_normal: &mut Vec3,
) {
    let position_local = in_position.extend(1.0);

    let position_world = mesh.model * position_local;
    let position_clip = view.view_proj * position_world;

    *out_clip_position = position_clip;
    *out_world_position = position_world;
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

#[spirv(fragment)]
#[allow(unused_variables)]
pub fn fragment_sdf(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] view: &View,
    #[spirv(uniform, descriptor_set = 0, binding = 9)] globals: &Globals,
    #[spirv(uniform, descriptor_set = 2, binding = 0)] mesh: &Mesh,
    #[spirv(position)] in_clip_position: Vec4,
    #[spirv(front_facing)] in_is_front: bool,
    in_world_position: Vec4,
    in_world_normal: Vec3,
    out_color: &mut Vec4,
) {
    let camera = view.view.col(3);
    let ray_delta = in_world_position - camera;
    let ray_dist = ray_delta.length();
    let ray_direction = ray_delta.normalize();
    let object = mesh.model.col(3);

    let inv_model_rot = Mat3::from_mat4(mesh.model).transpose();

    let mut near = 0.0;
    let mut far = 1000.0;

    if in_is_front {
        near = ray_dist;
    } else {
        far = ray_dist;
    }

    let sdf = Torus::default()
        .with((Torus::core, Circle::radius), 0.75)
        .with((Torus::shell, Circle::radius), 0.25);
    //let sdf = Sphere::default();
    //let sdf = Octahedron::default();
    //let sdf = Cube::default();

    let raymarcher = rust_gpu_sdf::prelude::SphereTraceLipschitz::default();

    let eye = inv_model_rot * (camera.truncate() - object.truncate());
    let dir = inv_model_rot * ray_direction.truncate();

    const EPSILON: f32 = 0.01;
    const MAX_STEPS: u32 = 150;

    let out = raymarcher.raymarch::<_, MAX_STEPS>(&sdf, near, far, eye, dir, EPSILON);

    let pos = eye + dir * out.closest;
    let pos = (camera + ray_direction * out.closest.max(EPSILON)).truncate();
    let inverse_transpose_rot = Mat3::from_mat4(mesh.inverse_transpose_model);

    let normal = inverse_transpose_rot
        * *TetrahedronNormal {
            target: TetrahedronGradient {
                sdf,
                epsilon: EPSILON,
            },
            ..default()
        }
        .evaluate((pos - object.truncate()) * 1000.0);

    let normal_remapped = normal * Vec3::splat(0.5) + Vec3::splat(0.5);
 
    *out_color = normal_remapped.extend(1.0);
 
    if !out.hit {
        let steps_norm = (out.steps as f32 / MAX_STEPS as f32).powf(4.0) * 30.0;
        *out_color *= steps_norm;
    }
}
