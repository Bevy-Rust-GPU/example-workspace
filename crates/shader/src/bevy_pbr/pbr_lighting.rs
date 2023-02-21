#[allow(unused_imports)]
use spirv_std::{
    glam::{Vec2, Vec3, Vec4},
    num_traits::{Float, FloatConst},
};

use crate::saturate::Saturate;

use super::mesh_view_types::{
    DirectionalLight, PointLight, POINT_LIGHT_FLAGS_SPOT_LIGHT_Y_NEGATIVE,
};

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
    return attenuation * 1.0 / distance_square.max(0.0001);
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
    let d = k * k * (1.0 / <f32 as FloatConst>::PI());
    return d;
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
    let v = 0.5 / (lambda_v + lambda_l);
    return v;
}

// Fresnel function
// see https://google.github.io/filament/Filament.html#citation-schlick94
// F_Schlick(v,h,f_0,f_90) = f_0 + (f_90 − f_0) (1 − v⋅h)^5
pub fn f_shlick_vec(f0: Vec3, f90: f32, voh: f32) -> Vec3 {
    // not using mix to keep the vec3 and float versions identical
    return f0 + (f90 - f0) * (1.0 - voh).powf(5.0);
}

pub fn f_schlick(f0: f32, f90: f32, voh: f32) -> f32 {
    // not using mix to keep the vec3 and float versions identical
    return f0 + (f90 - f0) * (1.0 - voh).powf(5.0);
}

pub fn fresnel(f0: Vec3, loh: f32) -> Vec3 {
    // f_90 suitable for ambient occlusion
    // see https://google.github.io/filament/Filament.html#lighting/occlusion
    let f90 = (f0.dot(Vec3::splat(50.0 * 0.33))).saturate();
    return f_shlick_vec(f0, f90, loh);
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

    return (specular_intensity * d * v) * f;
}

// Diffuse BRDF
// https://google.github.io/filament/Filament.html#materialsystem/diffusebrdf
// fd(v,l) = σ/π * 1 / { |n⋅v||n⋅l| } ∫Ω D(m,α) G(v,l,m) (v⋅m) (l⋅m) dm
//
// simplest approximation
// float Fd_Lambert() {
//     return 1.0 / PI;
// }
//
// vec3 Fd = diffuseColor * Fd_Lambert();
//
// Disney approximation
// See https://google.github.io/filament/Filament.html#citation-burley12
// minimal quality difference
pub fn fd_burley(roughness: f32, nov: f32, nol: f32, loh: f32) -> f32 {
    let f90 = 0.5 + 2.0 * roughness * loh * loh;
    let light_scatter = f_schlick(1.0, f90, nol);
    let view_scatter = f_schlick(1.0, f90, nov);
    return light_scatter * view_scatter * (1.0 / <f32 as FloatConst>::PI());
}

// From https://www.unrealengine.com/en-US/blog/physically-based-shading-on-mobile
pub fn env_brdf_approx(f0: Vec3, perceptual_roughness: f32, nov: f32) -> Vec3 {
    let c0 = Vec4::new(-1.0, -0.0275, -0.572, 0.022);
    let c1 = Vec4::new(1.0, 0.0425, 1.04, -0.04);
    let r = perceptual_roughness * c0 + c1;
    let a004 = (r.x * r.x).min((-9.28 * nov).exp2()) * r.x + r.y;
    let ab = Vec2::new(-1.04, 1.04) * a004 + Vec2::new(r.z, r.w);
    return f0 * ab.x + ab.y;
}

pub fn perceptual_roughness_to_roughness(perceptual_roughness: f32) -> f32 {
    // clamp perceptual roughness to prevent precision problems
    // According to Filament design 0.089 is recommended for mobile
    // Filament uses 0.045 for non-mobile
    let clamped_perceptual_roughness = perceptual_roughness.clamp(0.089, 1.0);
    return clamped_perceptual_roughness * clamped_perceptual_roughness;
}

pub fn point_light(
    world_position: Vec3,
    light: &PointLight,
    roughness: f32,
    n_dot_v: f32,
    n: Vec3,
    v: Vec3,
    r: Vec3,
    f0: Vec3,
    diffuse_color: Vec3,
) -> Vec3 {
    let light_to_frag = light.position_radius.truncate() - world_position;
    let distance_square = light_to_frag.dot(light_to_frag);
    let range_attenuation =
        get_distance_attenuation(distance_square, light.color_inverse_square_range.w);

    // Specular.
    // Representative Point Area Lights.
    // see http://blog.selfshadow.com/publications/s2013-shading-course/karis/s2013_pbs_epic_notes_v2.pdf p14-16
    let a = roughness;
    let center_to_ray = light_to_frag.dot(r) * r - light_to_frag;
    let closest_point = light_to_frag
        + center_to_ray
            * (light.position_radius.w * center_to_ray.dot(center_to_ray).sqrt().recip())
                .saturate();
    let l_spec_length_inverse = closest_point.dot(closest_point).sqrt().recip();
    let normalization_factor =
        a / (a + (light.position_radius.w * 0.5 * l_spec_length_inverse)).saturate();
    let specular_intensity = normalization_factor * normalization_factor;

    let l: Vec3 = closest_point * l_spec_length_inverse; // ().normalize() equivalent?
    let h: Vec3 = (l + v).normalize();
    let nol: f32 = n.dot(l).saturate();
    let noh: f32 = n.dot(h).saturate();
    let loh: f32 = l.dot(h).saturate();

    let specular_light = specular(f0, roughness, h, n_dot_v, nol, noh, loh, specular_intensity);

    // Diffuse.
    // Comes after specular since its NoL is used in the lighting equation.
    let l = light_to_frag.normalize();
    let h = (l + v).normalize();
    let nol = n.dot(l).saturate();
    let _noh = n.dot(h).saturate();
    let loh = l.dot(h).saturate();

    let diffuse = diffuse_color * fd_burley(roughness, n_dot_v, nol, loh);

    // See https://google.github.io/filament/Filament.html#mjx-eqn-pointLightLuminanceEquation
    // Lout = f(v,l) Φ / { 4 π d^2 }⟨n⋅l⟩
    // where
    // f(v,l) = (f_d(v,l) + f_r(v,l)) * light_color
    // Φ is luminous power in lumens
    // our rangeAttentuation = 1 / d^2 multiplied with an attenuation factor for smoothing at the edge of the non-physical maximum light radius

    // For a point light, luminous intensity, I, in lumens per steradian is given by:
    // I = Φ / 4 π
    // The derivation of this can be seen here: https://google.github.io/filament/Filament.html#mjx-eqn-pointLightLuminousPower

    // NOTE: light.color.rgb is premultiplied with light.intensity / 4 π (which would be the luminous intensity) on the CPU

    // TODO compensate for energy loss https://google.github.io/filament/Filament.html#materialsystem/improvingthebrdfs/energylossinspecularreflectance

    return ((diffuse + specular_light) * light.color_inverse_square_range.truncate())
        * (range_attenuation * nol);
}

pub fn spot_light(
    world_position: Vec3,
    light: &PointLight,
    roughness: f32,
    n_dot_v: f32,
    n: Vec3,
    v: Vec3,
    r: Vec3,
    f0: Vec3,
    diffuse_color: Vec3,
) -> Vec3 {
    // reuse the point light calculations
    let point_light = point_light(
        world_position,
        light,
        roughness,
        n_dot_v,
        n,
        v,
        r,
        f0,
        diffuse_color,
    );

    // reconstruct spot dir from x/z and y-direction flag
    let mut spot_dir = Vec3::new(light.light_custom_data.x, 0.0, light.light_custom_data.y);
    spot_dir.y = (0.0_f32.max(1.0 - spot_dir.x * spot_dir.x - spot_dir.z * spot_dir.z)).sqrt();
    if (light.flags & POINT_LIGHT_FLAGS_SPOT_LIGHT_Y_NEGATIVE) != 0 {
        spot_dir.y = -spot_dir.y;
    }
    let light_to_frag = light.position_radius.truncate() - world_position;

    // calculate attenuation based on filament formula https://google.github.io/filament/Filament.html#listing_glslpunctuallight
    // spot_scale and spot_offset have been precomputed
    // note we normalize here to get "l" from the filament listing. spot_dir is already normalized
    let cd = -spot_dir.dot(light_to_frag.normalize());
    let attenuation = (cd * light.light_custom_data.z + light.light_custom_data.w).saturate();
    let spot_attenuation = attenuation * attenuation;

    return point_light * spot_attenuation;
}

pub fn directional_light(
    light: DirectionalLight,
    roughness: f32,
    n_dot_v: f32,
    normal: Vec3,
    view: Vec3,
    r: Vec3,
    f0: Vec3,
    diffuse_color: Vec3,
) -> Vec3 {
    let incident_light = light.direction_to_light;

    let half_vector = (incident_light + view).normalize();
    let nol = (normal.dot(incident_light)).saturate();
    let noh = (normal.dot(half_vector)).saturate();
    let loh = (incident_light.dot(half_vector)).saturate();

    let diffuse = diffuse_color * fd_burley(roughness, n_dot_v, nol, loh);
    let specular_intensity = 1.0;
    let specular_light = specular(
        f0,
        roughness,
        half_vector,
        n_dot_v,
        nol,
        noh,
        loh,
        specular_intensity,
    );

    return (specular_light + diffuse) * light.color.truncate() * nol;
}
