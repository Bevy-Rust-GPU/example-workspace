use spirv_std::glam::{Vec2, Vec3};

// from https://64.github.io/tonemapping/
// reinhard on RGB oversaturates colors
pub fn tonemapping_reinhard(color: Vec3) -> Vec3 {
    color / (1.0 + color)
}

pub fn tonemapping_reinhard_extended(color: Vec3, max_white: f32) -> Vec3 {
    let numerator = color * (1.0 + (color / Vec3::splat(max_white * max_white)));
    numerator / (1.0 + color)
}

// luminance coefficients from Rec. 709.
// https://en.wikipedia.org/wiki/Rec._709
pub fn tonemapping_luminance(v: Vec3) -> f32 {
    v.dot(Vec3::new(0.2126, 0.7152, 0.0722))
}

pub fn tonemapping_change_luminance(c_in: Vec3, l_out: f32) -> Vec3 {
    let l_in = tonemapping_luminance(c_in);
    c_in * (l_out / l_in)
}

pub fn reinhard_luminance(color: Vec3) -> Vec3 {
    let l_old = tonemapping_luminance(color);
    let l_new = l_old / (1.0 + l_old);
    tonemapping_change_luminance(color, l_new)
}

// Source: Advanced VR Rendering, GDC 2015, Alex Vlachos, Valve, Slide 49
// https://media.steampowered.com/apps/valve/2015/Alex_Vlachos_Advanced_VR_Rendering_GDC2015.pdf
pub fn screen_space_dither(frag_coord: Vec2) -> Vec3 {
    let mut dither = Vec3::splat(Vec2::new(171.0, 231.0).dot(frag_coord));
    dither = (dither / Vec3::new(103.0, 71.0, 97.0)).fract();
    (dither - 0.5) / 255.0
}

