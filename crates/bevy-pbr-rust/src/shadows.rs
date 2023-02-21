use spirv_std::Image;

#[cfg(feature = "no_array_textures_support")]
pub type PointShadowTextures = Image!(cube, type = f32, depth = true);

#[cfg(not(feature = "no_array_textures_support"))]
pub type PointShadowTextures = Image!(cube, type = f32, depth = true, arrayed = true);

#[cfg(feature = "no_array_textures_support")]
pub type DirectionalShadowTextures = Image!(2D, type = f32, depth = true);

#[cfg(not(feature = "no_array_textures_support"))]
pub type DirectionalShadowTextures = Image!(2D, type = f32, depth = true, arrayed = true);
