use spirv_std::{
    glam::{Vec2, Vec3},
    Image, Sampler,
};

pub trait DirectionalShadowTextures {
    fn sample_depth_reference(
        &self,
        sampler: &Sampler,
        shadow_uv: Vec2,
        depth: f32,
        light_id: u32,
        spot_light_shadowmap_offset: i32,
    ) -> f32;
}

pub type DirectionalShadowTexture = Image!(2D, type = f32, depth = true);

impl DirectionalShadowTextures for DirectionalShadowTexture {
    fn sample_depth_reference(
        &self,
        sampler: &Sampler,
        shadow_uv: Vec2,
        depth: f32,
        _: u32,
        _: i32,
    ) -> f32 {
        self.sample_depth_reference(*sampler, shadow_uv, depth)
    }
}

pub type DirectionalShadowTextureArray = Image!(2D, type = f32, depth = true, arrayed = true);

impl DirectionalShadowTextures for DirectionalShadowTextureArray {
    fn sample_depth_reference(
        &self,
        sampler: &Sampler,
        shadow_uv: Vec2,
        depth: f32,
        light_id: u32,
        spot_light_shadowmap_offset: i32,
    ) -> f32 {
        self.sample_depth_reference_by_lod(
            *sampler,
            shadow_uv.extend(0.0),
            depth,
            light_id as f32 + spot_light_shadowmap_offset as f32,
        )
    }
}

pub trait PointShadowTextures {
    fn sample_depth_reference(
        &self,
        sampler: &Sampler,
        frag_ls: Vec3,
        depth: f32,
        light_id: u32,
    ) -> f32;
}

pub type PointShadowTexture = Image!(cube, type = f32, depth = true);

impl PointShadowTextures for PointShadowTexture {
    fn sample_depth_reference(
        &self,
        sampler: &Sampler,
        frag_ls: Vec3,
        depth: f32,
        _: u32,
    ) -> f32 {
        self.sample_depth_reference(*sampler, frag_ls, depth)
    }
}

pub type PointShadowTextureArray = Image!(cube, type = f32, depth = true, arrayed = true);

impl PointShadowTextures for PointShadowTextureArray {
    fn sample_depth_reference(
        &self,
        sampler: &Sampler,
        frag_ls: Vec3,
        depth: f32,
        light_id: u32,
    ) -> f32 {
        self.sample_depth_reference_by_lod(*sampler, frag_ls.extend(1.0), depth, light_id as f32)
    }
}
