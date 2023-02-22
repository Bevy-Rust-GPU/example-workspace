use spirv_std::glam::Vec4;

pub trait Dither {
    fn dither(in_frag_coord: Vec4, output_color: Vec4) -> Vec4;
}

pub enum DebandDither {}

impl Dither for () {
    fn dither(_: Vec4, output_color: Vec4) -> Vec4 {
        output_color
    }
}

impl Dither for DebandDither {
    fn dither(in_frag_coord: Vec4, output_color: Vec4) -> Vec4 {
        let mut output_rgb = output_color.truncate();
        output_rgb = output_rgb.powf(1.0 / 2.2);
        output_rgb = output_rgb
            + crate::prelude::screen_space_dither(in_frag_coord.truncate().truncate());
        // This conversion back to linear space is required because our output texture format is
        // SRGB; the GPU will assume our output is linear and will apply an SRGB conversion.
        output_rgb = output_rgb.powf(2.2);
        output_rgb.extend(output_color.w)
    }
}

