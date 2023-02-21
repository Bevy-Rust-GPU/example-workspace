use spirv_std::{
    arch::unsigned_min,
    glam::{UVec3, Vec2, Vec4},
};

#[allow(unused_imports)]
use spirv_std::num_traits::Float;

use super::mesh_view_types::{Lights, View};

// this must match CLUSTER_COUNT_SIZE in light.rs
pub const CLUSTER_COUNT_SIZE: u32 = 9;

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

    output_color
}
