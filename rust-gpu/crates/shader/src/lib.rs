#![no_std]
#![feature(asm_experimental_arch)]

use spirv_std::{
    spirv,
    glam::{UVec3, IVec2, Vec4}, Image,
};

fn hash(value: u32) -> u32 {
    let mut state = value;
    state = state ^ 2747636419;
    state = state * 2654435769;
    state = state ^ state >> 16;
    state = state * 2654435769;
    state = state ^ state >> 16;
    state = state * 2654435769;
    return state;
}

fn randomFloat(value: u32) -> f32 {
    return (hash(value) as f32) / 4294967295.0;
}

pub type Image_2D_SNORM =  Image!(2D, format=rgba8_snorm, sampled=false);

fn is_alive(location: IVec2, offset_x: i32, offset_y: i32, image: &Image_2D_SNORM) -> i32 {
    let value= image.read(location + IVec2::new(offset_x, offset_y));
    return value.x as i32;
}

fn count_alive(location: IVec2, image: &Image_2D_SNORM) -> i32 {
    return is_alive(location, -1, -1, image) +
           is_alive(location, -1,  0, image) +
           is_alive(location, -1,  1, image) +
           is_alive(location,  0, -1, image) +
           is_alive(location,  0,  1, image) +
           is_alive(location,  1, -1, image) +
           is_alive(location,  1,  0, image) +
           is_alive(location,  1,  1, image);
}



#[spirv(compute(threads(8,8)))]
pub fn init(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(num_workgroups)] num: UVec3,
    #[spirv(descriptor_set = 0, binding = 0)] texture: &Image_2D_SNORM,
) {

    let coord = IVec2::new(id.x as i32, id.y as i32);
    let randomNumber = randomFloat(id.y * num.x + id.x);
    let alive = randomNumber > 0.9;
    let alive_f = alive as i32 as f32;
    let pixel = Vec4::new(alive_f, alive_f, alive_f, 1.0);
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

    let coord = IVec2::new(id.x as i32, id.y as i32);
    let n_alive = count_alive(coord, texture);
    let alive = n_alive == 3 || n_alive == 2 && is_alive(coord, 0, 0, texture) == 1;
    let alive_f = alive as i32 as f32;
    let pixel = Vec4::new(alive_f, alive_f, alive_f, 1.0);

    unsafe { spirv_std::arch::workgroup_memory_barrier_with_group_sync() };

    unsafe {
        texture.write(coord, pixel);
    }

}