use spirv_std::glam::Vec4;

pub trait Tonemapper {
    fn tonemap(c: Vec4) -> Vec4;
}

impl Tonemapper for () {
    fn tonemap(c: Vec4) -> Vec4 {
        c
    }
}

pub enum TonemapInShader {}

impl Tonemapper for TonemapInShader {
    fn tonemap(output_color: Vec4) -> Vec4 {
        // tone_mapping
        crate::prelude::reinhard_luminance(output_color.truncate()).extend(output_color.w)

        // Gamma correction.
        // Not needed with sRGB buffer
        // output_color.rgb = pow(output_color.rgb, vec3(1.0 / 2.2));
    }
}

