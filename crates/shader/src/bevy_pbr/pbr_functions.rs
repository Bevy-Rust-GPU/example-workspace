use spirv_std::{
    glam::{Vec2, Vec3, Vec4},
    Sampler,
};

use crate::reflect::Reflect;

use super::{
    clustered_forward::cluster_debug_visualization,
    mesh_types::{Mesh, MESH_FLAGS_SHADOW_RECEIVER_BIT},
    mesh_view_types::{
        ClusterLightIndexLists, ClusterOffsetsAndCounts, Lights, PointLights, View,
        DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT, POINT_LIGHT_FLAGS_SHADOWS_ENABLED_BIT,
    },
    pbr_lighting::{env_brdf_approx, perceptual_roughness_to_roughness},
    pbr_types::StandardMaterial,
    shadows::{DirectionalShadowTextures, PointShadowTextures},
};

pub fn prepare_world_normal(world_normal: Vec3, double_sided: bool, is_front: bool) -> Vec3 {
    let output: Vec3 = world_normal;

    // NOTE: When NOT using normal-mapping, if looking at the back face of a double-sided
    // material, the normal needs to be inverted. This is a branchless version of that.
    #[cfg(all(
        not(feature = "VERTEX_TANGENTS"),
        not(feature = "STANDARDMATERIAL_NORMAL_MAP")
    ))]
    let output = (if !double_sided || is_front { 1.0 } else { 0.0 } * 2.0 - 1.0) * output;

    output
}

pub fn apply_normal_mapping(
    _standard_material_flags: u32,
    world_normal: Vec3,

    #[cfg(all(feature = "VERTEX_TANGENTS", feature = "STANDARDMATERIAL_NORMAL_MAP"))]
    world_tangent: Vec4,

    #[cfg(feature = "VERTEX_UVS")] _uv: Vec2,
) -> Vec3 {
    // NOTE: The mikktspace method of normal mapping explicitly requires that the world normal NOT
    // be re-normalized in the fragment shader. This is primarily to match the way mikktspace
    // bakes vertex tangents and normal maps so that this is the exact inverse. Blender, Unity,
    // Unreal Engine, Godot, and more all use the mikktspace method. Do not change this code
    // unless you really know what you are doing.
    // http://www.mikktspace.com/
    let n = world_normal;

    #[cfg(all(feature = "VERTEX_TANGENTS", feature = "STANDARDMATERIAL_NORMAL_MAP"))]
    {
        // NOTE: The mikktspace method of normal mapping explicitly requires that these NOT be
        // normalized nor any Gram-Schmidt applied to ensure the vertex normal is orthogonal to the
        // vertex tangent! Do not change this code unless you really know what you are doing.
        // http://www.mikktspace.com/
        let T: Vec3 = world_tangent.xyz;
        let B: Vec3 = world_tangent.w * cross(N, T);
    }

    #[cfg(all(
        feature = "VERTEX_TANGENTS",
        feature = "VERTEX_UVS",
        feaure = "STANDARDMATERIAL_NORMAL_MAP"
    ))]
    {
        // Nt is the tangent-space normal.
        let mut Nt = textureSample(normal_map_texture, normal_map_sampler, uv).rgb;
        if ((standard_material_flags & STANDARD_MATERIAL_FLAGS_TWO_COMPONENT_NORMAL_MAP) != 0u) {
            // Only use the xy components and derive z for 2-component normal maps.
            Nt = Vec3(Nt.rg * 2.0 - 1.0, 0.0);
            Nt.z = sqrt(1.0 - Nt.x * Nt.x - Nt.y * Nt.y);
        } else {
            Nt = Nt * 2.0 - 1.0;
        }
        // Normal maps authored for DirectX require flipping the y component
        if ((standard_material_flags & STANDARD_MATERIAL_FLAGS_FLIP_NORMAL_MAP_Y) != 0u) {
            Nt.y = -Nt.y;
        }
        // NOTE: The mikktspace method of normal mapping applies maps the tangent-space normal from
        // the normal map texture in this way to be an EXACT inverse of how the normal map baker
        // calculates the normal maps so there is no error introduced. Do not change this code
        // unless you really know what you are doing.
        // http://www.mikktspace.com/
        N = Nt.x * T + Nt.y * B + Nt.z * N;
    }

    n.normalize()
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
    pub fn pbr(
        &self,
        view: &View,
        mesh: &Mesh,
        lights: &Lights,
        point_lights: &PointLights,
        cluster_light_index_lists: &ClusterLightIndexLists,
        cluster_offsets_and_counts: &ClusterOffsetsAndCounts,
        directional_shadow_textures: &DirectionalShadowTextures,
        directional_shadow_textures_sampler: &Sampler,
        point_shadow_textures: &PointShadowTextures,
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

            #[cfg(feature = "NO_STORAGE_BUFFERS_SUPPORT")]
            let light = &point_lights.data[light_id as usize];

            #[cfg(not(feature = "NO_STORAGE_BUFFERS_SUPPORT"))]
            let light = unsafe { point_lights.data.index(light_id as usize) };

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

            #[cfg(feature = "NO_STORAGE_BUFFERS_SUPPORT")]
            let light = &point_lights.data[light_id as usize];

            #[cfg(not(feature = "NO_STORAGE_BUFFERS_SUPPORT"))]
            let light = unsafe { point_lights.data.index(light_id as usize) };

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

        output_color = cluster_debug_visualization(
            output_color,
            view_z,
            self.is_orthographic,
            offset_and_counts,
            cluster_index,
        );

        output_color
    }
}

#[cfg(feature = "TONEMAP_IN_SHADER")]
pub fn tone_mapping(input: Vec4) -> Vec4 {
    use crate::bevy_core_pipeline::tonemapping_shared::reinhard_luminance;

    // tone_mapping
    reinhard_luminance(input.truncate()).extend(input.w)

    // Gamma correction.
    // Not needed with sRGB buffer
    // output_color.rgb = pow(output_color.rgb, vec3(1.0 / 2.2));
}

#[cfg(feature = "DEBAND_DITHER")]
pub fn dither(color: Vec4, pos: Vec2) -> Vec4 {
    use crate::bevy_core_pipeline::tonemapping_shared::screen_space_dither;

    (color.truncate() + screen_space_dither(pos)).extend(color.w)
}
