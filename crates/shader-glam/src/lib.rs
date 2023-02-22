#![no_std]

#[cfg(not(feature = "spirv-std"))]
pub use glam;

#[cfg(feature = "spirv-std")]
pub use spirv_std::glam;
