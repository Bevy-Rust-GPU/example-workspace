#![no_std]
#![feature(asm_experimental_arch)]

use bevy_pbr::pbr_types::StandardMaterial;

pub mod bevy_core_pipeline;
pub mod bevy_pbr;
pub mod reflect;
pub mod saturate;

#[repr(C)]
pub struct BaseMaterial {
    base: StandardMaterial,
}

