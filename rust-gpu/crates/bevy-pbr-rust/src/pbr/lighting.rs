#[allow(unused_imports)]
use spirv_std::{
    glam::{Vec2, Vec3, Vec4},
    num_traits::{Float, FloatConst},
};

use rust_gpu_util::prelude::Saturate;

// From the Filament design doc
// https://google.github.io/filament/Filament.html#table_symbols
// Symbol Definition
// v    View unit vector
// l    Incident light unit vector
// n    Surface normal unit vector
// h    Half unit vector between l and v
// f    BRDF
// f_d    Diffuse component of a BRDF
// f_r    Specular component of a BRDF
// α    Roughness, remapped from using input perceptualRoughness
// σ    Diffuse reflectance
// Ω    Spherical domain
// f0    Reflectance at normal incidence
// f90    Reflectance at grazing angle
// χ+(a)    Heaviside function (1 if a>0 and 0 otherwise)
// nior    Index of refraction (IOR) of an interface
// ⟨n⋅l⟩    Dot product clamped to [0..1]
// ⟨a⟩    Saturated value (clamped to [0..1])

// The Bidirectional Reflectance Distribution Function (BRDF) describes the surface response of a standard material
// and consists of two components, the diffuse component (f_d) and the specular component (f_r):
// f(v,l) = f_d(v,l) + f_r(v,l)
//
// The form of the microfacet model is the same for diffuse and specular
// f_r(v,l) = f_d(v,l) = 1 / { |n⋅v||n⋅l| } ∫_Ω D(m,α) G(v,l,m) f_m(v,l,m) (v⋅m) (l⋅m) dm
//
// In which:
// D, also called the Normal Distribution Function (NDF) models the distribution of the microfacets
// G models the visibility (or occlusion or shadow-masking) of the microfacets
// f_m is the microfacet BRDF and differs between specular and diffuse components
//
// The above integration needs to be approximated.

// distanceAttenuation is simply the square falloff of light intensity
// combined with a smooth attenuation at the edge of the light radius
//
// light radius is a non-physical construct for efficiency purposes,
// because otherwise every light affects every fragment in the scene
pub fn get_distance_attenuation(distance_square: f32, inverse_range_squared: f32) -> f32 {
    let factor = distance_square * inverse_range_squared;
    let smooth_factor = (1.0 - factor * factor).saturate();
    let attenuation = smooth_factor * smooth_factor;
    attenuation * 1.0 / distance_square.max(0.0001)
}

// Normal distribution function (specular D)
// Based on https://google.github.io/filament/Filament.html#citation-walter07

// D_GGX(h,α) = α^2 / { π ((n⋅h)^2 (α2−1) + 1)^2 }

// Simple implementation, has precision problems when using fp16 instead of fp32
// see https://google.github.io/filament/Filament.html#listing_speculardfp16
pub fn d_ggx(roughness: f32, noh: f32, _h: Vec3) -> f32 {
    let one_minus_noh_squared = 1.0 - noh * noh;
    let a = noh * roughness;
    let k = roughness / (one_minus_noh_squared + a * a);
    k * k * (1.0 / <f32 as FloatConst>::PI())
}

// Visibility function (Specular G)
// V(v,l,a) = G(v,l,α) / { 4 (n⋅v) (n⋅l) }
// such that f_r becomes
// f_r(v,l) = D(h,α) V(v,l,α) F(v,h,f0)
// where
// V(v,l,α) = 0.5 / { n⋅l sqrt((n⋅v)^2 (1−α2) + α2) + n⋅v sqrt((n⋅l)^2 (1−α2) + α2) }
// Note the two sqrt's, that may be slow on mobile, see https://google.github.io/filament/Filament.html#listing_approximatedspecularv
pub fn v_smith_ggx_correlated(roughness: f32, nov: f32, nol: f32) -> f32 {
    let a2 = roughness * roughness;
    let lambda_v = nol * ((nov - a2 * nov) * nov + a2).sqrt();
    let lambda_l = nov * ((nol - a2 * nol) * nol + a2).sqrt();
    0.5 / (lambda_v + lambda_l)
}

// Fresnel function
// see https://google.github.io/filament/Filament.html#citation-schlick94
// F_Schlick(v,h,f_0,f_90) = f_0 + (f_90 − f_0) (1 − v⋅h)^5
pub fn f_shlick_vec(f0: Vec3, f90: f32, voh: f32) -> Vec3 {
    // not using mix to keep the vec3 and float versions identical
    f0 + (f90 - f0) * (1.0 - voh).powf(5.0)
}

pub fn f_schlick(f0: f32, f90: f32, voh: f32) -> f32 {
    // not using mix to keep the vec3 and float versions identical
    f0 + (f90 - f0) * (1.0 - voh).powf(5.0)
}

pub fn fresnel(f0: Vec3, loh: f32) -> Vec3 {
    // f_90 suitable for ambient occlusion
    // see https://google.github.io/filament/Filament.html#lighting/occlusion
    let f90 = (f0.dot(Vec3::splat(50.0 * 0.33))).saturate();
    f_shlick_vec(f0, f90, loh)
}

// Specular BRDF
// https://google.github.io/filament/Filament.html#materialsystem/specularbrdf

// Cook-Torrance approximation of the microfacet model integration using Fresnel law F to model f_m
// f_r(v,l) = { D(h,α) G(v,l,α) F(v,h,f0) } / { 4 (n⋅v) (n⋅l) }
pub fn specular(
    f0: Vec3,
    roughness: f32,
    h: Vec3,
    nov: f32,
    nol: f32,
    noh: f32,
    loh: f32,
    specular_intensity: f32,
) -> Vec3 {
    let d = d_ggx(roughness, noh, h);
    let v = v_smith_ggx_correlated(roughness, nov, nol);
    let f = fresnel(f0, loh);

    (specular_intensity * d * v) * f
}

// Diffuse BRDF
// https://google.github.io/filament/Filament.html#materialsystem/diffusebrdf
// fd(v,l) = σ/π * 1 / { |n⋅v||n⋅l| } ∫Ω D(m,α) G(v,l,m) (v⋅m) (l⋅m) dm
//
// simplest approximation
// fn fd_lambert() -> f32 {
//     1.0 / PI
// }
//
// let fd = diffuse_color * fd_lambert();
//
// Disney approximation
// See https://google.github.io/filament/Filament.html#citation-burley12
// minimal quality difference
pub fn fd_burley(roughness: f32, nov: f32, nol: f32, loh: f32) -> f32 {
    let f90 = 0.5 + 2.0 * roughness * loh * loh;
    let light_scatter = f_schlick(1.0, f90, nol);
    let view_scatter = f_schlick(1.0, f90, nov);
    light_scatter * view_scatter * (1.0 / <f32 as FloatConst>::PI())
}

// From https://www.unrealengine.com/en-US/blog/physically-based-shading-on-mobile
pub fn env_brdf_approx(f0: Vec3, perceptual_roughness: f32, nov: f32) -> Vec3 {
    let c0 = Vec4::new(-1.0, -0.0275, -0.572, 0.022);
    let c1 = Vec4::new(1.0, 0.0425, 1.04, -0.04);
    let r = perceptual_roughness * c0 + c1;
    let a004 = (r.x * r.x).min((-9.28 * nov).exp2()) * r.x + r.y;
    let ab = Vec2::new(-1.04, 1.04) * a004 + Vec2::new(r.z, r.w);
    f0 * ab.x + ab.y
}

pub fn perceptual_roughness_to_roughness(perceptual_roughness: f32) -> f32 {
    // clamp perceptual roughness to prevent precision problems
    // According to Filament design 0.089 is recommended for mobile
    // Filament uses 0.045 for non-mobile
    let clamped_perceptual_roughness = perceptual_roughness.clamp(0.089, 1.0);
    clamped_perceptual_roughness * clamped_perceptual_roughness
}
