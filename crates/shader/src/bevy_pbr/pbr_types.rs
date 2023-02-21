use spirv_std::glam::Vec4;

pub const STANDARD_MATERIAL_FLAGS_BASE_COLOR_TEXTURE_BIT: u32 = 1;
pub const STANDARD_MATERIAL_FLAGS_EMISSIVE_TEXTURE_BIT: u32 = 2;
pub const STANDARD_MATERIAL_FLAGS_METALLIC_ROUGHNESS_TEXTURE_BIT: u32 = 4;
pub const STANDARD_MATERIAL_FLAGS_OCCLUSION_TEXTURE_BIT: u32 = 8;
pub const STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT: u32 = 16;
pub const STANDARD_MATERIAL_FLAGS_UNLIT_BIT: u32 = 32;
pub const STANDARD_MATERIAL_FLAGS_ALPHA_MODE_OPAQUE: u32 = 64;
pub const STANDARD_MATERIAL_FLAGS_ALPHA_MODE_MASK: u32 = 128;
pub const STANDARD_MATERIAL_FLAGS_ALPHA_MODE_BLEND: u32 = 256;
pub const STANDARD_MATERIAL_FLAGS_TWO_COMPONENT_NORMAL_MAP: u32 = 512;
pub const STANDARD_MATERIAL_FLAGS_FLIP_NORMAL_MAP_Y: u32 = 1024;

#[repr(C)]
pub struct StandardMaterial {
    pub base_color: Vec4,
    pub emissive: Vec4,
    pub perceptual_roughness: f32,
    pub metallic: f32,
    pub reflectance: f32,
    // 'flags' is a bit field indicating various options. u32 is 32 bits so we have up to 32 options.
    pub flags: u32,
    pub alpha_cutoff: f32,
}

impl Default for StandardMaterial {
    fn default() -> Self {
        StandardMaterial {
            base_color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            emissive: Vec4::new(0.0, 0.0, 0.0, 1.0),
            perceptual_roughness: 0.089,
            metallic: 0.01,
            reflectance: 0.5,
            flags: STANDARD_MATERIAL_FLAGS_ALPHA_MODE_OPAQUE,
            alpha_cutoff: 0.5,
        }
    }
}

