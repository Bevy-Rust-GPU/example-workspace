use spirv_std::{
    glam::{Mat3, Vec2, Vec3, Vec4},
    Image, Sampler,
};

#[allow(unused_imports)]
use spirv_std::num_traits::Float;

use super::mesh_view_types::{Lights, PointLights, POINT_LIGHT_FLAGS_SPOT_LIGHT_Y_NEGATIVE};

#[cfg(feature = "NO_ARRAY_TEXTURES_SUPPORT")]
pub type PointShadowTextures = Image!(cube, type = f32, depth = true);

#[cfg(not(feature = "NO_ARRAY_TEXTURES_SUPPORT"))]
pub type PointShadowTextures = Image!(cube, type = f32, depth = true, arrayed = true);

pub fn fetch_point_shadow(
    point_lights: &PointLights,
    point_shadow_textures: &PointShadowTextures,
    point_shadow_textures_sampler: &Sampler,
    light_id: u32,
    frag_position: Vec4,
    surface_normal: Vec3,
) -> f32 {
    #[cfg(feature = "NO_STORAGE_BUFFERS_SUPPORT")]
    let light = point_lights.data[light_id as usize];

    #[cfg(not(feature = "NO_STORAGE_BUFFERS_SUPPORT"))]
    let light = unsafe { point_lights.data.index(light_id as usize) };

    // because the shadow maps align with the axes and the frustum planes are at 45 degrees
    // we can get the worldspace depth by taking the largest absolute axis
    let surface_to_light = light.position_radius.truncate() - frag_position.truncate();
    let surface_to_light_abs = surface_to_light.abs();
    let distance_to_light = surface_to_light_abs
        .x
        .max(surface_to_light_abs.y.max(surface_to_light_abs.z));

    // The normal bias here is already scaled by the texel size at 1 world unit from the light.
    // The texel size increases proportionally with distance from the light so multiplying by
    // distance to light scales the normal bias to the texel size at the fragment distance.
    let normal_offset = light.shadow_normal_bias * distance_to_light * surface_normal;
    let depth_offset = light.shadow_depth_bias * surface_to_light.normalize();
    let offset_position = frag_position.truncate() + normal_offset + depth_offset;

    // similar largest-absolute-axis trick as above, but now with the offset fragment position
    let frag_ls = light.position_radius.truncate() - offset_position;
    let abs_position_ls = frag_ls.abs();
    let major_axis_magnitude = abs_position_ls
        .x
        .max(abs_position_ls.y.max(abs_position_ls.z));

    // NOTE: These simplifications come from multiplying:
    // projection * vec4(0, 0, -major_axis_magnitude, 1.0)
    // and keeping only the terms that have any impact on the depth.
    // Projection-agnostic approach:
    let zw = -major_axis_magnitude * light.light_custom_data.truncate().truncate()
        + Vec2::new(light.light_custom_data.z, light.light_custom_data.w);
    let depth = zw.x / zw.y;

    // do the lookup, using HW PCF and comparison
    // NOTE: Due to the non-uniform control flow above, we must use the Level variant of
    // textureSampleCompare to avoid undefined behaviour due to some of the fragments in
    // a quad (2x2 fragments) being processed not being sampled, and this messing with
    // mip-mapping functionality. The shadow maps have no mipmaps so Level just samples
    // from LOD 0.
    #[cfg(feature = "NO_ARRAY_TEXTURES_SUPPORT")]
    {
        point_shadow_textures.sample_depth_reference(
            *point_shadow_textures_sampler,
            frag_ls.extend(1.0),
            depth,
        )
    }

    #[cfg(not(feature = "NO_ARRAY_TEXTURES_SUPPORT"))]
    {
        point_shadow_textures.sample_depth_reference_by_lod(
            *point_shadow_textures_sampler,
            frag_ls.extend(1.0),
            depth,
            light_id as f32,
        )
    }
}

#[cfg(feature = "NO_ARRAY_TEXTURES_SUPPORT")]
pub type DirectionalShadowTextures = Image!(2D, type = f32, depth = true);

#[cfg(not(feature = "NO_ARRAY_TEXTURES_SUPPORT"))]
pub type DirectionalShadowTextures = Image!(2D, type = f32, depth = true, arrayed = true);

pub fn fetch_spot_shadow(
    lights: &Lights,
    point_lights: &PointLights,
    directional_shadow_textures: &DirectionalShadowTextures,
    directional_shadow_textures_sampler: &Sampler,
    light_id: u32,
    frag_position: Vec4,
    surface_normal: Vec3,
) -> f32 {
    #[cfg(feature = "NO_STORAGE_BUFFERS_SUPPORT")]
    let light = point_lights.data[light_id as usize];

    #[cfg(not(feature = "NO_STORAGE_BUFFERS_SUPPORT"))]
    let light = unsafe { point_lights.data.index(light_id as usize) };

    let surface_to_light = light.position_radius.truncate() - frag_position.truncate();

    // construct the light view matrix
    let mut spot_dir = Vec3::new(light.light_custom_data.x, 0.0, light.light_custom_data.y);
    // reconstruct spot dir from x/z and y-direction flag
    spot_dir.y = (1.0_f32 - spot_dir.x * spot_dir.x - spot_dir.z * spot_dir.z).sqrt();
    if (light.flags & POINT_LIGHT_FLAGS_SPOT_LIGHT_Y_NEGATIVE) != 0 {
        spot_dir.y = -spot_dir.y;
    }

    // view matrix z_axis is the reverse of transform.forward()
    let fwd = -spot_dir;
    let distance_to_light = fwd.dot(surface_to_light);
    let offset_position = -surface_to_light
        + (light.shadow_depth_bias * surface_to_light.normalize())
        + (surface_normal * light.shadow_normal_bias) * distance_to_light;

    // the construction of the up and right vectors needs to precisely mirror the code
    // in render/light.rs:spot_light_view_matrix
    let mut sign = -1.0;
    if fwd.z >= 0.0 {
        sign = 1.0;
    }
    let a = -1.0 / (fwd.z + sign);
    let b = fwd.x * fwd.y * a;
    let up_dir = Vec3::new(1.0 + sign * fwd.x * fwd.x * a, sign * b, -sign * fwd.x);
    let right_dir = Vec3::new(-b, -sign - fwd.y * fwd.y * a, fwd.y);
    let light_inv_rot = Mat3 {
        x_axis: right_dir,
        y_axis: up_dir,
        z_axis: fwd,
    };

    // because the matrix is a pure rotation matrix, the inverse is just the transpose, and to calculate
    // the product of the transpose with a vector we can just post-multiply instead of pre-multplying.
    // this allows us to keep the matrix construction code identical between CPU and GPU.
    let projected_position = light_inv_rot * offset_position;

    // divide xy by perspective matrix "f" and by -projected.z (projected.z is -projection matrix's w)
    // to get ndc coordinates
    let f_div_minus_z = 1.0 / (light.spot_light_tan_angle * -projected_position.z);
    let shadow_xy_ndc = projected_position.truncate() * f_div_minus_z;
    // convert to uv coordinates
    let shadow_uv = shadow_xy_ndc * Vec2::new(0.5, -0.5) + Vec2::new(0.5, 0.5);

    // 0.1 must match POINT_LIGHT_NEAR_Z
    let depth = 0.1 / -projected_position.z;

    #[cfg(feature = "NO_ARRAY_TEXTURES_SUPPORT")]
    {
        textureSampleCompare(
            directional_shadow_textures,
            directional_shadow_textures_sampler,
            shadow_uv,
            depth,
        )
    }

    #[cfg(not(feature = "NO_ARRAY_TEXTURES_SUPPORT"))]
    {
        directional_shadow_textures.sample_depth_reference_by_lod(
            *directional_shadow_textures_sampler,
            shadow_uv.extend(0.0),
            depth,
            light_id as f32 + lights.spot_light_shadowmap_offset as f32,
        )
    }
}

pub fn fetch_directional_shadow(
    lights: &Lights,
    directional_shadow_textures: &DirectionalShadowTextures,
    directional_shadow_textures_sampler: &Sampler,
    light_id: u32,
    frag_position: Vec4,
    surface_normal: Vec3,
) -> f32 {
    let light = lights.directional_lights[light_id as usize];

    // The normal bias is scaled to the texel size.
    let normal_offset = light.shadow_normal_bias * surface_normal;
    let depth_offset = light.shadow_depth_bias * light.direction_to_light;
    let offset_position =
        (frag_position.truncate() + normal_offset + depth_offset).extend(frag_position.w);

    let offset_position_clip = light.view_projection * offset_position;
    if offset_position_clip.w <= 0.0 {
        return 1.0;
    }
    let offset_position_ndc = offset_position_clip.truncate() / offset_position_clip.w;
    // No shadow outside the orthographic projection volume
    if (offset_position_ndc.x < -1.0 || offset_position_ndc.y < -1.0)
        || offset_position_ndc.z < 0.0
        || (offset_position_ndc.x > 1.0
            || offset_position_ndc.y > 1.0
            || offset_position_ndc.z > 1.0)
    {
        return 1.0;
    }

    // compute texture coordinates for shadow lookup, compensating for the Y-flip difference
    // between the NDC and texture coordinates
    let flip_correction = Vec2::new(0.5, -0.5);
    let light_local = offset_position_ndc.truncate() * flip_correction + Vec2::new(0.5, 0.5);

    let depth = offset_position_ndc.z;
    // do the lookup, using HW PCF and comparison
    // NOTE: Due to non-uniform control flow above, we must use the level variant of the texture
    // sampler to avoid use of implicit derivatives causing possible undefined behavior.
    #[cfg(feature = "NO_ARRAY_TEXTURES_SUPPORT")]
    {
        directional_shadow_textures.sample_depth_reference(
            *directional_shadow_textures_sampler,
            light_local.extend(0.0),
            depth,
        )
    }

    #[cfg(not(feature = "NO_ARRAY_TEXTURES_SUPPORT"))]
    {
        directional_shadow_textures.sample_depth_reference_by_lod(
            *directional_shadow_textures_sampler,
            light_local.extend(0.0),
            depth,
            light_id as f32,
        )
    }
}
