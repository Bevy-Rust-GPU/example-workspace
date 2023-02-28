pub mod bindings;
pub mod dither;
pub mod entry_points;
pub mod lighting;
pub mod standard_material;
pub mod tonemapper;

use spirv_std::{
    glam::{Vec3, Vec4},
    Sampler,
};

use rust_gpu_util::prelude::Reflect;

use crate::prelude::{
    env_brdf_approx, perceptual_roughness_to_roughness, ClusterDebugVisualization,
    ClusterLightIndexLists, ClusterOffsetsAndCounts, DirectionalShadowTextures, Lights, Mesh,
    PointLights, PointShadowTextures, StandardMaterial, View,
    DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT, MESH_FLAGS_SHADOW_RECEIVER_BIT,
    POINT_LIGHT_FLAGS_SHADOWS_ENABLED_BIT,
};

#[repr(C)]
pub struct BaseMaterial {
    pub base: StandardMaterial,
}

#[repr(C)]
pub struct PbrInput {
    pub material: StandardMaterial,
    pub occlusion: f32,
    pub frag_coord: Vec4,
    pub world_position: Vec4,
    // Normalized world normal used for shadow mapping as normal-mapping is not used for shadow
    // mapping
    pub world_normal: Vec3,
    // Normalized normal-mapped world normal used for lighting
    pub n: Vec3,
    // Normalized view vector in world space, pointing from the fragment world position toward the
    // view world position
    pub v: Vec3,
    pub is_orthographic: bool,
}

impl Default for PbrInput {
    fn default() -> Self {
        PbrInput {
            material: StandardMaterial::default(),
            occlusion: 1.0,

            frag_coord: Vec4::new(0.0, 0.0, 0.0, 1.0),
            world_position: Vec4::new(0.0, 0.0, 0.0, 1.0),
            world_normal: Vec3::new(0.0, 0.0, 1.0),

            is_orthographic: false,

            n: Vec3::new(0.0, 0.0, 1.0),
            v: Vec3::new(1.0, 0.0, 0.0),
        }
    }
}

impl PbrInput {
    pub fn pbr<
        PL: PointLights,
        DS: DirectionalShadowTextures,
        PS: PointShadowTextures,
        CL: ClusterLightIndexLists,
        CO: ClusterOffsetsAndCounts,
        CD: ClusterDebugVisualization,
    >(
        &self,
        view: &View,
        mesh: &Mesh,
        lights: &Lights,
        point_lights: &PL,
        cluster_light_index_lists: &CL,
        cluster_offsets_and_counts: &CO,
        directional_shadow_textures: &DS,
        directional_shadow_textures_sampler: &Sampler,
        point_shadow_textures: &PS,
        point_shadow_textures_sampler: &Sampler,
    ) -> Vec4 {
        let mut output_color = self.material.base_color;

        // TODO use .a for exposure compensation in HDR
        let emissive = self.material.emissive;

        // calculate non-linear roughness from linear perceptualRoughness
        let metallic = self.material.metallic;
        let perceptual_roughness = self.material.perceptual_roughness;
        let roughness = perceptual_roughness_to_roughness(perceptual_roughness);

        let occlusion = self.occlusion;

        output_color = self.material.alpha_discard(output_color);

        // Neubelt and Pettineo 2013, "Crafting a Next-gen Material Pipeline for The Order: 1886"
        let n_dot_v = self.n.dot(self.v).max(0.0001);

        // Remapping [0,1] reflectance to F0
        // See https://google.github.io/filament/Filament.html#materialsystem/parameterization/remapping
        let reflectance = self.material.reflectance;
        let f0 = 0.16 * reflectance * reflectance * (1.0 - metallic)
            + output_color.truncate() * metallic;

        // Diffuse strength inversely related to metallicity
        let diffuse_color = output_color.truncate() * (1.0 - metallic);

        let r = -self.v.reflect(self.n);

        // accumulate color
        let mut light_accum: Vec3 = Vec3::ZERO;

        let view_z = Vec4::new(
            view.inverse_view.x_axis.z,
            view.inverse_view.y_axis.z,
            view.inverse_view.z_axis.z,
            view.inverse_view.w_axis.z,
        )
        .dot(self.world_position);
        let cluster_index = lights.fragment_cluster_index(
            view,
            self.frag_coord.truncate().truncate(),
            view_z,
            self.is_orthographic,
        );
        let offset_and_counts = cluster_offsets_and_counts.unpack(cluster_index);

        // point lights
        for i in offset_and_counts.x as u32..(offset_and_counts.x + offset_and_counts.y) as u32 {
            let light_id = cluster_light_index_lists.get_light_id(i);

            let light = &point_lights.get_point_light(light_id);

            let mut shadow: f32 = 1.0;
            if (mesh.flags & MESH_FLAGS_SHADOW_RECEIVER_BIT) != 0
                && (light.flags & POINT_LIGHT_FLAGS_SHADOWS_ENABLED_BIT) != 0
            {
                shadow = point_lights.fetch_point_shadow(
                    point_shadow_textures,
                    point_shadow_textures_sampler,
                    light_id,
                    self.world_position,
                    self.world_normal,
                );
            }
            let light_contrib = light.point_light(
                self.world_position.truncate(),
                roughness,
                n_dot_v,
                self.n,
                self.v,
                r,
                f0,
                diffuse_color,
            );
            light_accum = light_accum + light_contrib * shadow;
        }

        // spot lights
        for i in (offset_and_counts.x + offset_and_counts.y) as u32
            ..(offset_and_counts.x + offset_and_counts.y + offset_and_counts.z) as u32
        {
            let light_id = cluster_light_index_lists.get_light_id(i);

            let light = point_lights.get_point_light(light_id);

            let mut shadow: f32 = 1.0;
            if (mesh.flags & MESH_FLAGS_SHADOW_RECEIVER_BIT) != 0
                && (light.flags & POINT_LIGHT_FLAGS_SHADOWS_ENABLED_BIT) != 0
            {
                shadow = point_lights.fetch_spot_shadow(
                    lights,
                    directional_shadow_textures,
                    directional_shadow_textures_sampler,
                    light_id,
                    self.world_position,
                    self.world_normal,
                );
            }
            let light_contrib = light.spot_light(
                self.world_position.truncate(),
                roughness,
                n_dot_v,
                self.n,
                self.v,
                r,
                f0,
                diffuse_color,
            );
            light_accum = light_accum + light_contrib * shadow;
        }

        let n_directional_lights = lights.n_directional_lights;
        for i in 0..n_directional_lights {
            let light = lights.directional_lights[i as usize];
            let mut shadow: f32 = 1.0;
            if (mesh.flags & MESH_FLAGS_SHADOW_RECEIVER_BIT) != 0
                && (light.flags & DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT) != 0
            {
                shadow = lights.fetch_directional_shadow(
                    directional_shadow_textures,
                    directional_shadow_textures_sampler,
                    i,
                    self.world_position,
                    self.world_normal,
                );
            }
            let light_contrib =
                light.directional_light(roughness, n_dot_v, self.n, self.v, f0, diffuse_color);
            light_accum = light_accum + light_contrib * shadow;
        }

        let diffuse_ambient = env_brdf_approx(diffuse_color, 1.0, n_dot_v);
        let specular_ambient = env_brdf_approx(f0, perceptual_roughness, n_dot_v);

        output_color = (light_accum
            + (diffuse_ambient + specular_ambient) * lights.ambient_color.truncate() * occlusion
            + emissive.truncate() * output_color.w)
            .extend(output_color.w);

        output_color = CD::cluster_debug_visualization(
            lights,
            output_color,
            view_z,
            self.is_orthographic,
            offset_and_counts,
            cluster_index,
        );

        output_color
    }
}
