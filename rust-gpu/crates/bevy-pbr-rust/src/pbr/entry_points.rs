use bevy_rust_gpu_macros::permutate;
use spirv_std::{
    glam::{Vec2, Vec3, Vec4},
    macros::spirv,
    Sampler,
};

use crate::prelude::{
    BaseColorTexture, BaseMaterialNormalMap, ClusterDebugVisualization, ClusterLightIndexLists,
    ClusterLightIndexListsStorage, ClusterLightIndexListsUniform, ClusterOffsetsAndCounts,
    ClusterOffsetsAndCountsStorage, ClusterOffsetsAndCountsUniform, DebandDither, DebugZSlices,
    DirectionalShadowTexture, DirectionalShadowTextureArray, DirectionalShadowTextures, Dither,
    EmissiveTexture, Lights, Mesh, MetallicRoughnessTexture, NormalMapTexture, OcclusionTexture,
    PbrInput, PointLights, PointLightsStorage, PointLightsUniform, PointShadowTexture,
    PointShadowTextureArray, PointShadowTextures, SkinnedMesh, Skinning, StandardMaterialNormalMap,
    TonemapInShader, Tonemapper, VertexColor, VertexNormal, VertexPosition, VertexTangent,
    VertexUv, View, STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT, STANDARD_MATERIAL_FLAGS_UNLIT_BIT,
};

use super::BaseMaterial;

#[permutate(
    mappings = {
        texture: texture | array,
        buffer: uniform | storage,
        position: position | none,
        normal: normal | none,
        uv: uv | none,
        tangent: tangent | none,
        color: color | none,
        normal_map: normal_map | none,
        skinned: skinned | none,
        tonemap: tonemap | none,
        deband: deband | none,
        cluster_debug: debug_z_slices | debug_cluster_light_complexity | debug_cluster_coherence | none,
    },
    permutations = [
        (array, uniform, position, normal, uv, none, none, none, none, tonemap, deband, none),
        (array, uniform, position, normal, uv, tangent, none, none, none, tonemap, deband, none),
        (array, uniform, position, normal, uv, tangent, color, none, none, tonemap, deband, none),
    ]
)]
#[spirv(fragment)]
#[allow(non_snake_case)]
pub fn fragment(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] view: &View,
    #[spirv(uniform, descriptor_set = 0, binding = 1)] lights: &Lights,

    #[permutate(texture = texture)]
    #[spirv(descriptor_set = 0, binding = 2)]
    point_shadow_textures: &PointShadowTexture,

    #[permutate(texture = array)]
    #[spirv(descriptor_set = 0, binding = 2)]
    point_shadow_textures: &PointShadowTextureArray,

    #[spirv(descriptor_set = 0, binding = 3)] point_shadow_textures_sampler: &Sampler,

    #[permutate(texture = texture)]
    #[spirv(descriptor_set = 0, binding = 4)]
    directional_shadow_textures: &DirectionalShadowTexture,

    #[permutate(texture = array)]
    #[spirv(descriptor_set = 0, binding = 4)]
    directional_shadow_textures: &DirectionalShadowTextureArray,

    #[spirv(descriptor_set = 0, binding = 5)] directional_shadow_textures_sampler: &Sampler,

    #[permutate(buffer = uniform)]
    #[spirv(uniform, descriptor_set = 0, binding = 6)]
    point_lights: &PointLightsUniform,

    #[permutate(buffer = storage)]
    #[spirv(storage_buffer, descriptor_set = 0, binding = 6)]
    point_lights: &PointLightsStorage,

    #[permutate(buffer = uniform)]
    #[spirv(uniform, descriptor_set = 0, binding = 7)]
    cluster_light_index_lists: &ClusterLightIndexListsUniform,

    #[permutate(buffer = storage)]
    #[spirv(storage_buffer, descriptor_set = 0, binding = 7)]
    cluster_light_index_lists: &ClusterLightIndexListsStorage,

    #[permutate(buffer = uniform)]
    #[spirv(uniform, descriptor_set = 0, binding = 8)]
    cluster_offsets_and_counts: &ClusterOffsetsAndCountsUniform,

    #[permutate(buffer = storage)]
    #[spirv(storage_buffer, descriptor_set = 0, binding = 8)]
    cluster_offsets_and_counts: &ClusterOffsetsAndCountsStorage,

    #[spirv(uniform, descriptor_set = 1, binding = 0)] material: &BaseMaterial,
    #[spirv(descriptor_set = 1, binding = 1)] base_color_texture: &BaseColorTexture,
    #[spirv(descriptor_set = 1, binding = 2)] base_color_sampler: &Sampler,
    #[spirv(descriptor_set = 1, binding = 3)] emissive_texture: &EmissiveTexture,
    #[spirv(descriptor_set = 1, binding = 4)] emissive_sampler: &Sampler,
    #[spirv(descriptor_set = 1, binding = 5)] metallic_roughness_texture: &MetallicRoughnessTexture,
    #[spirv(descriptor_set = 1, binding = 6)] metallic_roughness_sampler: &Sampler,
    #[spirv(descriptor_set = 1, binding = 7)] occlusion_texture: &OcclusionTexture,
    #[spirv(descriptor_set = 1, binding = 8)] occlusion_sampler: &Sampler,
    #[spirv(descriptor_set = 1, binding = 9)] normal_map_texture: &NormalMapTexture,
    #[spirv(descriptor_set = 1, binding = 10)] normal_map_sampler: &Sampler,

    #[spirv(uniform, descriptor_set = 2, binding = 0)] mesh: &Mesh,

    #[spirv(front_facing)] in_is_front: bool,
    #[spirv(position)] in_frag_coord: Vec4,
    in_world_position: Vec4,
    in_world_normal: Vec3,
    in_uv: Vec2,
    #[permutate(tangent = tangent)] in_tangent: Vec4,
    #[permutate(color = color)] in_color: Vec4,
    output_color: &mut Vec4,
) {
    #[permutate(texture = texture)]
    type _PointShadow = PointShadowTexture;
    #[permutate(texture = array)]
    type _PointShadow = PointShadowTextureArray;

    #[permutate(texture = texture)]
    type _DirectionalShadow = DirectionalShadowTexture;
    #[permutate(texture = array)]
    type _DirectionalShadow = DirectionalShadowTextureArray;

    #[permutate(buffer = uniform)]
    type _PointLights = PointLightsUniform;
    #[permutate(buffer = storage)]
    type _PointLights = PointLightsStorage;

    #[permutate(buffer = uniform)]
    type _ClusterLightIndexLists = ClusterLightIndexListsUniform;
    #[permutate(buffer = storage)]
    type _ClusterLightIndexLists = ClusterLightIndexListsStorage;

    #[permutate(buffer = uniform)]
    type _ClusterOffsetsAndCounts = ClusterOffsetsAndCountsUniform;
    #[permutate(buffer = storage)]
    type _ClusterOffsetsAndCounts = ClusterOffsetsAndCountsStorage;

    #[permutate(position = position)]
    type _Position = Vec4;
    #[permutate(position = none)]
    type _Position = ();

    #[permutate(normal = normal)]
    type _Normal = Vec3;
    #[permutate(normal = none)]
    type _Normal = ();

    #[permutate(uv = uv)]
    type _Uv = Vec2;
    #[permutate(uv = none)]
    type _Uv = ();

    #[permutate(tangent = tangent)]
    type _Tangent = Vec4;
    #[permutate(tangent = none)]
    type _Tangent = ();

    #[permutate(color = color)]
    type _Color = Vec4;
    #[permutate(color = none)]
    type _Color = ();

    #[permutate(normal_map = normal_map)]
    type _NormalMap = StandardMaterialNormalMap;
    #[permutate(normal_map = none)]
    type _NormalMap = ();

    #[permutate(skinned = skinned)]
    type _Skinned = SkinnedMesh;
    #[permutate(skinned = none)]
    type _Skinned = ();

    #[permutate(tonemap = tonemap)]
    type _Tonemap = TonemapInShader;
    #[permutate(tonemap = none)]
    type _Tonemap = ();

    #[permutate(deband = deband)]
    type _Deband = DebandDither;
    #[permutate(deband = none)]
    type _Deband = ();

    #[permutate(cluster_debug = debug_z_slices)]
    type _ClusterDebug = DebugZSlices;
    #[permutate(cluster_debug = debug_cluster_light_complexity)]
    type _ClusterDebug = DebugClusterLightComplexity;
    #[permutate(cluster_debug = debug_cluster_coherence)]
    type _ClusterDebug = DebugClusterCoherence;
    #[permutate(cluster_debug = none)]
    type _ClusterDebug = ();

    fragment_impl::<
        _PointShadow,
        _DirectionalShadow,
        _PointLights,
        _ClusterLightIndexLists,
        _ClusterOffsetsAndCounts,
        _Position,
        _Normal,
        _Uv,
        _Tangent,
        _Color,
        _NormalMap,
        _Skinned,
        _Tonemap,
        _Deband,
        _ClusterDebug,
    >(
        view,
        lights,
        point_shadow_textures,
        point_shadow_textures_sampler,
        directional_shadow_textures,
        directional_shadow_textures_sampler,
        point_lights,
        cluster_light_index_lists,
        cluster_offsets_and_counts,
        material,
        base_color_texture,
        base_color_sampler,
        emissive_texture,
        emissive_sampler,
        metallic_roughness_texture,
        metallic_roughness_sampler,
        occlusion_texture,
        occlusion_sampler,
        normal_map_texture,
        normal_map_sampler,
        mesh,
        in_is_front,
        in_frag_coord,
        #[permutate(position = position)]
        &in_world_position,
        #[permutate(position = none)]
        &(),
        #[permutate(normal = normal)]
        &in_world_normal,
        #[permutate(normal = none)]
        &(),
        #[permutate(uv = uv)]
        &in_uv,
        #[permutate(uv = none)]
        &(),
        #[permutate(tangent = tangent)]
        &in_tangent,
        #[permutate(tangent = none)]
        &(),
        #[permutate(color = color)]
        &in_color,
        #[permutate(color = none)]
        &(),
        output_color,
    )
}

pub fn fragment_impl<
    PS: PointShadowTextures,
    DS: DirectionalShadowTextures,
    PL: PointLights,
    CL: ClusterLightIndexLists,
    CO: ClusterOffsetsAndCounts,
    VP: VertexPosition,
    VN: VertexNormal,
    VU: VertexUv,
    VT: VertexTangent,
    VC: VertexColor,
    N: BaseMaterialNormalMap,
    SM: Skinning,
    TM: Tonemapper,
    DT: Dither,
    CD: ClusterDebugVisualization,
>(
    view: &View,
    lights: &Lights,
    point_shadow_textures: &PS,
    point_shadow_textures_sampler: &Sampler,
    directional_shadow_textures: &DS,
    directional_shadow_textures_sampler: &Sampler,
    point_lights: &PL,
    cluster_light_index_lists: &CL,
    cluster_offsets_and_counts: &CO,

    material: &BaseMaterial,
    base_color_texture: &BaseColorTexture,
    base_color_sampler: &Sampler,
    emissive_texture: &EmissiveTexture,
    emissive_sampler: &Sampler,
    metallic_roughness_texture: &MetallicRoughnessTexture,
    metallic_roughness_sampler: &Sampler,
    occlusion_texture: &OcclusionTexture,
    occlusion_sampler: &Sampler,
    normal_map_texture: &NormalMapTexture,
    normal_map_sampler: &Sampler,

    mesh: &Mesh,

    in_is_front: bool,
    in_frag_coord: Vec4,

    vertex_position: &VP,
    vertex_normal: &VN,
    vertex_uv: &VU,
    vertex_tangent: &VT,
    vertex_color: &VC,

    output_color: &mut Vec4,
) {
    *output_color = material.base.base_color;

    *output_color = vertex_color.apply(*output_color);

    vertex_uv.sample_base_color_texture(
        base_color_texture,
        base_color_sampler,
        material,
        output_color,
    );

    // NOTE: Unlit bit not set means == 0 is true, so the true case is if lit
    if material.base.flags & STANDARD_MATERIAL_FLAGS_UNLIT_BIT == 0 {
        // Prepare a 'processed' StandardMaterial by sampling all textures to resolve
        // the material members
        let mut pbr_input = PbrInput::default();

        pbr_input.material.base_color = *output_color;
        pbr_input.material.reflectance = material.base.reflectance;
        pbr_input.material.flags = material.base.flags;
        pbr_input.material.alpha_cutoff = material.base.alpha_cutoff;

        // TODO use .a for exposure compensation in HDR
        let mut emissive = material.base.emissive;

        vertex_uv.sample_emissive_texture(
            emissive_texture,
            emissive_sampler,
            material,
            &mut emissive,
        );

        pbr_input.material.emissive = emissive;

        let mut metallic = material.base.metallic;
        let mut perceptual_roughness = material.base.perceptual_roughness;

        vertex_uv.sample_metallic_roughness_texture(
            metallic_roughness_texture,
            metallic_roughness_sampler,
            material,
            &mut metallic,
            &mut perceptual_roughness,
        );

        pbr_input.material.metallic = metallic;
        pbr_input.material.perceptual_roughness = perceptual_roughness;

        let mut occlusion: f32 = 1.0;

        vertex_uv.sample_occlusion_texture(
            occlusion_texture,
            occlusion_sampler,
            material,
            &mut occlusion,
        );

        pbr_input.occlusion = occlusion;

        pbr_input.frag_coord = in_frag_coord;
        vertex_position.apply_pbr_position(&mut pbr_input);
        vertex_normal.prepare_world_normal::<VT, N>(
            (material.base.flags & STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0,
            in_is_front,
            &mut pbr_input,
        );

        pbr_input.is_orthographic = view.projection.w_axis.w == 1.0;

        let pn = pbr_input.world_normal;
        pn.apply_pbr_input_n::<VU, VT, N>(
            material.base.flags,
            vertex_uv,
            vertex_tangent,
            normal_map_texture,
            normal_map_sampler,
            &mut pbr_input,
        );

        vertex_position.apply_pbr_v(view, &mut pbr_input);

        *output_color = pbr_input.pbr::<PL, DS, PS, CL, CO, CD>(
            view,
            mesh,
            lights,
            point_lights,
            cluster_light_index_lists,
            cluster_offsets_and_counts,
            directional_shadow_textures,
            directional_shadow_textures_sampler,
            point_shadow_textures,
            point_shadow_textures_sampler,
        );
    } else {
        *output_color = material.base.alpha_discard(*output_color);
    }

    *output_color = TM::tonemap(*output_color);
    *output_color = DT::dither(in_frag_coord, *output_color);
}
