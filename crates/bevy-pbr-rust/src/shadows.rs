use spirv_std::Image;

#[cfg(feature = "NO_ARRAY_TEXTURES_SUPPORT")]
pub type PointShadowTextures = Image!(cube, type = f32, depth = true);

#[cfg(not(feature = "NO_ARRAY_TEXTURES_SUPPORT"))]
pub type PointShadowTextures = Image!(cube, type = f32, depth = true, arrayed = true);

#[cfg(feature = "NO_ARRAY_TEXTURES_SUPPORT")]
pub type DirectionalShadowTextures = Image!(2D, type = f32, depth = true);

#[cfg(not(feature = "NO_ARRAY_TEXTURES_SUPPORT"))]
pub type DirectionalShadowTextures = Image!(2D, type = f32, depth = true, arrayed = true);
