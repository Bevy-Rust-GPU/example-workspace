#![no_std]

pub mod reflect;
pub mod saturate;
pub mod smooth_step;

pub mod prelude;

use spirv_std::glam::Vec3;

#[allow(unused_imports)]
use spirv_std::num_traits::Float;

pub fn hsv2rgb(hue: f32, saturation: f32, value: f32) -> Vec3 {
    let rgb = ((((hue * 6.0 + Vec3::new(0.0, 4.0, 2.0)) % 6.0) - 3.0).abs() - 1.0)
        .clamp(Vec3::ZERO, Vec3::ONE);

    Vec3::ONE.lerp(rgb, saturation) * value
}

pub fn random_1d(s: f32) -> f32 {
    return ((s * 12.9898).sin() * 43758.5453123).fract();
}
