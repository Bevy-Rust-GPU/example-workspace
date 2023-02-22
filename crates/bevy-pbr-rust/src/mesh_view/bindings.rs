use spirv_std::Image;

pub type TextureDepthCube = Image!(cube, type = f32, sampled = true, depth = true);
pub type TextureDepthCubeArray =
    Image!(cube, type = f32, sampled = true, depth = true, arrayed = true);

pub type TextureDepth2d = Image!(2D, type = f32, sampled = true, depth = true);
pub type TextureDepth2dArray = Image!(2D, type = f32, sampled = true, depth = true, arrayed = true);
