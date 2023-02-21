use spirv_std::glam::Mat4;

#[repr(C)]
pub struct Mesh {
    pub model: Mat4,
    pub inverse_transpose_model: Mat4,
    // 'flags' is a bit field indicating various options. u32 is 32 bits so we have up to 32 options.
    pub flags: u32,
}

#[cfg(feature = "SKINNED")]
#[repr(C)]
pub struct SkinnedMesh {
    pub data: [Mat4; 256u],
}

pub const MESH_FLAGS_SHADOW_RECEIVER_BIT: u32 = 1;
// 2^31 - if the flag is set, the sign is positive, else it is negative
pub const MESH_FLAGS_SIGN_DETERMINANT_MODEL_3X3_BIT: u32 = 2147483648;

