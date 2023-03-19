#![no_std]
#![feature(asm_experimental_arch)]

use core::arch::asm;

pub use bevy_pbr_rust;

use bevy_pbr_rust::{
    prelude::{
        DepthPrepassTexture, Globals, Mesh, TextureDepth2d, TextureDepth2dArray,
        TextureDepthMultisampled2d, View,
    },
    tonemapping_shared::screen_space_dither,
};
use rust_gpu_sdf::{
    default,
    prelude::{
        uvs::{SdfUvs, TriplanarUvs},
        Capsule, CentralDiffGradient, CentralDiffNormal, ChebyshevMetric, Circle, Cube, Isosurface,
        Normal, Octahedron, Plane, Point, Sphere, Square, Sweep, TetrahedronGradient,
        TetrahedronNormal, Torus, Twist, Uv,
    },
    raymarch::Raymarch,
    signed_distance_field::DistanceFunction,
    type_fields::field::Field,
};
use spirv_std::{
    arch::kill,
    glam::{IVec2, Mat3, Quat, Vec2, Vec3, Vec4, Vec4Swizzles},
    spirv, Sampler,
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
    #[spirv(frag_coord)] in_clip_position: Vec4,
    in_world_normal: Vec3,
    out_color: &mut Vec4,
) {
    *out_color = in_world_normal.extend(1.0);
}

#[spirv(vertex)]
pub fn vertex_sdf(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] view: &View,
    #[spirv(uniform, descriptor_set = 2, binding = 0)] mesh: &Mesh,
    #[spirv(uniform, descriptor_set = 0, binding = 9)] globals: &Globals,

    in_position: Vec3,
    in_normal: Vec3,

    #[spirv(position)] out_clip_position: &mut Vec4,
    out_clip_position_2: &mut Vec4,
    out_world_position: &mut Vec4,
    out_world_normal: &mut Vec3,
) {
    let position_local = in_position.extend(1.0);

    let position_world = mesh.model * position_local;
    let position_clip = view.view_proj * position_world;

    *out_clip_position = position_clip;
    *out_clip_position_2 = position_clip;
    *out_world_position = position_world;
    *out_world_normal = in_normal;
}

#[spirv(fragment)]
#[allow(unused_variables)]
pub fn fragment_sdf(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] view: &View,
    #[spirv(uniform, descriptor_set = 0, binding = 9)] globals: &Globals,
    #[spirv(descriptor_set = 0, binding = 16)] depth_prepass_texture: &TextureDepth2d,
    #[spirv(uniform, descriptor_set = 2, binding = 0)] mesh: &Mesh,
    #[spirv(frag_coord)] in_frag_coord: Vec4,
    #[spirv(front_facing)] in_is_front: bool,
    in_clip_position: Vec4,
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

    enum DepthMode {
        World,
        Prepass,
    }

    let depth_mode = DepthMode::World;
    let depth = match depth_mode {
        DepthMode::World => ray_dist,
        DepthMode::Prepass => {
            let mut deproj_pos = view.inverse_projection
                * in_clip_position
                    .xy()
                    .extend(depth_prepass_texture.read(in_frag_coord.xy().as_ivec2()).x)
                    .extend(1.0);
            deproj_pos.z /= deproj_pos.w;
            deproj_pos.xyz().length()
        }
    };

    if in_is_front {
        near = depth;
    } else {
        far = depth;
    }

    const EPSILON: f32 = 0.006;
    const MAX_STEPS: u32 = 400;

    /*
    let sdf = Torus::default()
        .with((Torus::core, Circle::radius), 0.75)
        .with((Torus::shell, Circle::radius), 0.25);
    */

    let sdf = Sweep::<Circle, Square>::default()
        .with((Sweep::core, Circle::radius), 0.75)
        .with((Sweep::shell, Square::extent), Vec2::ONE * 0.25);

    //let sdf = Sphere::default();
    //let sdf = Octahedron::default();
    //let sdf = Cube::default();
    //let sdf = Plane::default();
    //let sdf = Capsule::<Vec3>::default();
    //let sdf = Isosurface::<ChebyshevMetric>::default();

    //let sdf = TetrahedronNormal::default().with(TetrahedronNormal::sdf, sdf);

    let raymarcher = rust_gpu_sdf::prelude::SphereTraceLipschitz::<MAX_STEPS>::default();

    let eye = inv_model_rot * (camera.truncate() - object.truncate());
    let dir = inv_model_rot * ray_direction.truncate();

    let out = raymarcher.raymarch::<_>(&sdf, near, far, eye, dir, EPSILON);

    let pos = eye + dir * out.closest_t;
    let inverse_transpose_rot = Mat3::from_mat4(mesh.inverse_transpose_model);

    let normal: Normal<Vec3> = sdf.evaluate(pos);
    let normal = inverse_transpose_rot * *normal;

    let normal_remapped = normal * Vec3::splat(0.5) + Vec3::splat(0.5);
    *out_color = normal_remapped.extend(1.0);

    let uv: Uv = sdf.evaluate(pos);
    *out_color = uv.extend(0.0).extend(1.0);

    if !out.hit {
        let steps_norm = (out.steps as f32 / MAX_STEPS as f32).powf(4.0) * 400.0;
        *out_color *= steps_norm;
    }
}
