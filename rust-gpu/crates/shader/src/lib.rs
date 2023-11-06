#![no_std]
#![feature(asm_experimental_arch)]

// use alloc::vec::Vec;
// pub use bevy_pbr_rust;

// use rust_gpu_bridge::glam;

// use bevy_pbr_rust::prelude::{Globals, Mesh, TextureDepth2d, View};
// use permutate_macro::permutate;
// use rust_gpu_bridge::{Mix, Mod, SmoothStep};

use spirv_std::{
    arch::{ddx, ddy, memory_barrier},
    glam::{Mat3, Vec2, Vec3, Vec4, Vec4Swizzles, Vec3Swizzles, Vec2Swizzles,UVec2},
    spirv,
    glam::{IVec4, UVec3, UVec4}, Image, image::StorageImage2d,
};

// #[allow(unused_imports)]
// use spirv_std::num_traits::Float;

// #[spirv(vertex)]
// pub fn vertex_warp(
//     #[spirv(uniform, descriptor_set = 0, binding = 0)] view: &View,
//     #[spirv(uniform, descriptor_set = 2, binding = 0)] mesh: &Mesh,
//     #[spirv(uniform, descriptor_set = 0, binding = 9)] globals: &Globals,

//     in_position: Vec3,
//     in_normal: Vec3,

//     #[spirv(position)] out_clip_position: &mut Vec4,
//     out_world_normal: &mut Vec3,
// ) {
//     let mut position_local = in_position.extend(1.0);

//     position_local.x += position_local.x * position_local.z * globals.time.sin();
//     position_local.y += position_local.y * position_local.z * globals.time.cos();
//     position_local.z += position_local.z * globals.time.sin() * globals.time.cos();

//     let position_world = mesh.model * position_local;
//     let position_clip = view.view_proj * position_world;

//     *out_clip_position = position_clip;
//     *out_world_normal = in_normal;
// }

// #[spirv(fragment)]
// #[allow(unused_variables)]
// pub fn fragment_normal(
//     #[spirv(frag_coord)] in_clip_position: Vec4,
//     in_world_normal: Vec3,
//     out_color: &mut Vec4,
// ) {
//     *out_color = in_world_normal.extend(1.0) + Vec4::new(1.0,0.0,0.0,0.0);
// }

// pub fn collatz(mut n: u32) -> Option<u32> {
//     let mut i = 0;
//     if n == 0 {
//         return None;
//     }
//     while n != 1 {
//         n = if n % 2 == 0 {
//             n / 2
//         } else {
//             // Overflow? (i.e. 3*n + 1 > 0xffff_ffff)
//             if n >= 0x5555_5555 {
//                 return None;
//             }
//             // TODO: Use this instead when/if checked add/mul can work: n.checked_mul(3)?.checked_add(1)?
//             3 * n + 1
//         };
//         i += 1;
//     }
//     Some(i)
// }

#[spirv(compute(threads(8,8)))]
pub fn init(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(num_workgroups)] num: UVec3,
    #[spirv(descriptor_set = 0, binding = 0)] texture: &Image!(2D, format=rgba8_snorm, sampled=false),
) {
    // let index = (id.y * num.x) as usize + id.y as usize;
    // let img_size: UVec2 = texture.query_size();
    let coord = UVec2::new(id.x, id.y );
    let pixel = Vec4::new(0.01, 0.01, 0.01, 0.01);
    unsafe {
        texture.write(coord, pixel);
    }
}

#[spirv(compute(threads(8,8)))]
pub fn update(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(num_workgroups)] num: UVec3,
    #[spirv(descriptor_set = 0, binding = 0)] texture:   &Image!(2D, format=rgba8_snorm, sampled=false),
){
    // let index = (id.y * num.x) as usize + id.y as usize;
    // let img_size: UVec2 = texture.query_size();
    let coord = UVec2::new(id.x, id.y);
    use noise_perlin::perlin_2d;
    // let query_size: UVec2 = texture.query_size();

    let x = perlin_2d(coord.x as f32/100.0, coord.y as f32/1000.0);
    let pixel =  Vec4::new(x,x,x,1.0);

    // unsafe { spirv_std::arch::workgroup_memory_barrier_with_group_sync() };


    unsafe {
        texture.write(coord, pixel);
    }
}