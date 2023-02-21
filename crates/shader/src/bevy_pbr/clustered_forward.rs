use spirv_std::{
    arch::unsigned_min,
    glam::{UVec3, Vec2, Vec3, Vec4},
};

#[allow(unused_imports)]
use spirv_std::num_traits::Float;

use super::mesh_view_types::{ClusterLightIndexLists, ClusterOffsetsAndCounts, Lights, View};

const NATURAL_LOG_BASE: f32 = 2.718281828459;

// NOTE: Keep in sync with bevy_pbr/src/light.rs
pub fn view_z_to_z_slice(lights: &Lights, view_z: f32, is_orthographic: bool) -> u32 {
    let z_slice = if is_orthographic {
        // NOTE: view_z is correct in the orthographic case
        ((view_z - lights.cluster_factors.z) * lights.cluster_factors.w).floor() as u32
    } else {
        // NOTE: had to use -view_z to make it positive else log(negative) is nan
        ((-view_z).log(NATURAL_LOG_BASE) * lights.cluster_factors.z - lights.cluster_factors.w
            + 1.0) as u32
    };
    // NOTE: We use min as we may limit the far z plane used for clustering to be closeer than
    // the furthest thing being drawn. This means that we need to limit to the maximum cluster.
    unsigned_min(z_slice, lights.cluster_dimensions.z - 1)
}

pub fn fragment_cluster_index(
    view: &View,
    lights: &Lights,
    frag_coord: Vec2,
    view_z: f32,
    is_orthographic: bool,
) -> u32 {
    let xy = ((frag_coord - view.viewport.truncate().truncate())
        * lights.cluster_factors.truncate().truncate())
    .floor()
    .as_uvec2();
    let z_slice = view_z_to_z_slice(lights, view_z, is_orthographic);
    // NOTE: Restricting cluster index to avoid undefined behavior when accessing uniform buffer
    // arrays based on the cluster index.
    return unsigned_min(
        (xy.y * lights.cluster_dimensions.x + xy.x) * lights.cluster_dimensions.z + z_slice,
        lights.cluster_dimensions.w - 1,
    );
}

// this must match CLUSTER_COUNT_SIZE in light.rs
pub const CLUSTER_COUNT_SIZE: u32 = 9;

pub fn unpack_offset_and_counts(
    cluster_offsets_and_counts: &ClusterOffsetsAndCounts,
    cluster_index: u32,
) -> UVec3 {
    #[cfg(feature = "NO_STORAGE_BUFFERS_SUPPORT")]
    {
        let v = cluster_offsets_and_counts.data[(cluster_index >> 2) as usize];
        let i = cluster_index & ((1 << 2) - 1);
        let offset_and_counts = match i {
            0 => v.x,
            1 => v.y,
            2 => v.z,
            3 => v.w,
            _ => panic!(),
        };
        //  [ 31     ..     18 | 17      ..      9 | 8       ..     0 ]
        //  [      offset      | point light count | spot light count ]
        UVec3::new(
            (offset_and_counts >> (CLUSTER_COUNT_SIZE * 2))
                & ((1 << (32 - (CLUSTER_COUNT_SIZE * 2))) - 1),
            (offset_and_counts >> CLUSTER_COUNT_SIZE) & ((1 << CLUSTER_COUNT_SIZE) - 1),
            offset_and_counts & ((1 << CLUSTER_COUNT_SIZE) - 1),
        )
    }

    #[cfg(not(feature = "NO_STORAGE_BUFFERS_SUPPORT"))]
    {
        unsafe {
            cluster_offsets_and_counts
                .data
                .index(cluster_index as usize)
        }
        .truncate()
    }
}

pub fn get_light_id(cluster_light_index_lists: &ClusterLightIndexLists, index: u32) -> u32 {
    #[cfg(feature = "NO_STORAGE_BUFFERS_SUPPORT")]
    {
        // The index is correct but in cluster_light_index_lists we pack 4 u8s into a u32
        // This means the index into cluster_light_index_lists is index / 4
        let indices = cluster_light_index_lists.data[(index >> 4) as usize]
            [((index >> 2) & ((1 << 2) - 1)) as usize];
        // And index % 4 gives the sub-index of the u8 within the u32 so we shift by 8 * sub-index
        return (indices >> (8 * (index & ((1 << 2) - 1)))) & ((1 << 8) - 1);
    }

    #[cfg(not(feature = "NO_STORAGE_BUFFERS_SUPPORT"))]
    {
        return unsafe { *cluster_light_index_lists.data.index(index as usize) };
        //return cluster_light_index_lists.data[index as usize];
    }
}

#[allow(unused_variables)]
pub fn cluster_debug_visualization(
    output_color: Vec4,
    view_z: f32,
    is_orthographic: bool,
    offset_and_counts: UVec3,
    cluster_index: u32,
) -> Vec4 {
    // Cluster allocation debug (using 'over' alpha blending)
    #[cfg(feature = "CLUSTERED_FORWARD_DEBUG_Z_SLICES")]
    {
        // NOTE: This debug mode visualises the z-slices
        let cluster_overlay_alpha = 0.1;
        let mut z_slice: u32 = view_z_to_z_slice(view_z, is_orthographic);
        // A hack to make the colors alternate a bit more
        if ((z_slice & 1u) == 1u) {
            z_slice = z_slice + lights.cluster_dimensions.z / 2u;
        }
        let slice_color = hsv2rgb(
            f32(z_slice) / f32(lights.cluster_dimensions.z + 1u),
            1.0,
            0.5,
        );
        output_color = Vec4(
            (1.0 - cluster_overlay_alpha) * output_color.rgb + cluster_overlay_alpha * slice_color,
            output_color.a,
        );
    }

    #[cfg(feature = "CLUSTERED_FORWARD_DEBUG_CLUSTER_LIGHT_COMPLEXITY")]
    {
        // NOTE: This debug mode visualises the number of lights within the cluster that contains
        // the fragment. It shows a sort of lighting complexity measure.
        let cluster_overlay_alpha = 0.1;
        let max_light_complexity_per_cluster = 64.0;
        output_color.r = (1.0 - cluster_overlay_alpha) * output_color.r
            + cluster_overlay_alpha
                * smoothStep(
                    0.0,
                    max_light_complexity_per_cluster,
                    f32(offset_and_counts[1] + offset_and_counts[2]),
                );
        output_color.g = (1.0 - cluster_overlay_alpha) * output_color.g
            + cluster_overlay_alpha
                * (1.0
                    - smoothStep(
                        0.0,
                        max_light_complexity_per_cluster,
                        f32(offset_and_counts[1] + offset_and_counts[2]),
                    ));
    }

    #[cfg(feature = "CLUSTERED_FORWARD_DEBUG_CLUSTER_COHERENCY")]
    {
        // NOTE: Visualizes the cluster to which the fragment belongs
        let cluster_overlay_alpha = 0.1;
        let cluster_color = hsv2rgb(random1D(f32(cluster_index)), 1.0, 0.5);
        output_color = Vec4(
            (1.0 - cluster_overlay_alpha) * output_color.rgb
                + cluster_overlay_alpha * cluster_color,
            output_color.a,
        );
    }

    return output_color;
}
