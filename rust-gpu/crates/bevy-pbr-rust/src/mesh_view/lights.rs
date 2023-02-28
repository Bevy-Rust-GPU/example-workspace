use spirv_std::{
    arch::unsigned_min,
    glam::{UVec4, Vec2, Vec3, Vec4},
    Sampler,
};

#[allow(unused_imports)]
use spirv_std::num_traits::Float;

use rust_gpu_util::prelude::NaturalLog;

use crate::prelude::{DirectionalLight, DirectionalShadowTextures, View};

#[derive(Default, Copy, Clone, PartialEq)]
#[repr(C)]
pub struct Lights {
    // NOTE: this array size must be kept in sync with the constants defined in bevy_pbr/src/render/light.rs
    pub directional_lights: [DirectionalLight; 10],
    pub ambient_color: Vec4,
    // x/y/z dimensions and n_clusters in w
    pub cluster_dimensions: UVec4,
    // xy are vec2<f32>(cluster_dimensions.xy) / vec2<f32>(view.width, view.height)
    //
    // For perspective projections:
    // z is cluster_dimensions.z / log(far / near)
    // w is cluster_dimensions.z * log(near) / log(far / near)
    //
    // For orthographic projections:
    // NOTE: near and far are +ve but -z is infront of the camera
    // z is -near
    // w is cluster_dimensions.z / (-far - -near)
    pub cluster_factors: Vec4,
    pub n_directional_lights: u32,
    pub spot_light_shadowmap_offset: i32,
}

impl Lights {
    pub fn fetch_directional_shadow<DS: DirectionalShadowTextures>(
        &self,
        directional_shadow_textures: &DS,
        directional_shadow_textures_sampler: &Sampler,
        light_id: u32,
        frag_position: Vec4,
        surface_normal: Vec3,
    ) -> f32 {
        let light = &self.directional_lights[light_id as usize];

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
        directional_shadow_textures.sample_depth_reference(
            directional_shadow_textures_sampler,
            light_local,
            depth,
            light_id,
            0,
        )
    }

    // NOTE: Keep in sync with bevy_pbr/src/light.rs
    pub fn view_z_to_z_slice(&self, view_z: f32, is_orthographic: bool) -> u32 {
        let z_slice = if is_orthographic {
            // NOTE: view_z is correct in the orthographic case
            ((view_z - self.cluster_factors.z) * self.cluster_factors.w).floor() as u32
        } else {
            // NOTE: had to use -view_z to make it positive else log(negative) is nan
            ((-view_z).natural_log() * self.cluster_factors.z - self.cluster_factors.w + 1.0) as u32
        };
        // NOTE: We use min as we may limit the far z plane used for clustering to be closeer than
        // the furthest thing being drawn. This means that we need to limit to the maximum cluster.
        unsigned_min(z_slice, self.cluster_dimensions.z - 1)
    }

    pub fn fragment_cluster_index(
        &self,
        view: &View,
        frag_coord: Vec2,
        view_z: f32,
        is_orthographic: bool,
    ) -> u32 {
        let xy = ((frag_coord - view.viewport.truncate().truncate())
            * self.cluster_factors.truncate().truncate())
        .floor()
        .as_uvec2();
        let z_slice = self.view_z_to_z_slice(view_z, is_orthographic);
        // NOTE: Restricting cluster index to avoid undefined behavior when accessing uniform buffer
        // arrays based on the cluster index.
        unsigned_min(
            (xy.y * self.cluster_dimensions.x + xy.x) * self.cluster_dimensions.z + z_slice,
            self.cluster_dimensions.w - 1,
        )
    }
}
