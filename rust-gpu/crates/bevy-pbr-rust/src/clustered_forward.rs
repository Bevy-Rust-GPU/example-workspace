use crate::glam::{UVec3, Vec4};
use rust_gpu_util::{hsv2rgb, random_1d, smooth_step::SmoothStep};

#[allow(unused_imports)]
use spirv_std::num_traits::Float;

use crate::prelude::Lights;

// this must match CLUSTER_COUNT_SIZE in light.rs
pub const CLUSTER_COUNT_SIZE: u32 = 9;

// Cluster allocation debug (using 'over' alpha blending)
pub trait ClusterDebugVisualization {
    fn cluster_debug_visualization(
        lights: &Lights,
        output_color: Vec4,
        view_z: f32,
        is_orthographic: bool,
        offset_and_counts: UVec3,
        cluster_index: u32,
    ) -> Vec4;
}

impl ClusterDebugVisualization for () {
    fn cluster_debug_visualization(
        _: &Lights,
        output_color: Vec4,
        _: f32,
        _: bool,
        _: UVec3,
        _: u32,
    ) -> Vec4 {
        output_color
    }
}

pub enum DebugZSlices {}

impl ClusterDebugVisualization for DebugZSlices {
    fn cluster_debug_visualization(
        lights: &Lights,
        output_color: Vec4,
        view_z: f32,
        is_orthographic: bool,
        _: UVec3,
        _: u32,
    ) -> Vec4 {
        // NOTE: This debug mode visualises the z-slices
        let cluster_overlay_alpha = 0.1;
        let mut z_slice: u32 = lights.view_z_to_z_slice(view_z, is_orthographic);
        // A hack to make the colors alternate a bit more
        if (z_slice & 1) == 1 {
            z_slice = z_slice + lights.cluster_dimensions.z / 2;
        }
        let slice_color = hsv2rgb(
            z_slice as f32 / (lights.cluster_dimensions.z + 1) as f32,
            1.0,
            0.5,
        );

        ((1.0 - cluster_overlay_alpha) * output_color.truncate()
            + cluster_overlay_alpha * slice_color)
            .extend(output_color.w)
    }
}

pub enum DebugClusterLightComplexity {}

impl ClusterDebugVisualization for DebugClusterLightComplexity {
    fn cluster_debug_visualization(
        _: &Lights,
        mut output_color: Vec4,
        _: f32,
        _: bool,
        offset_and_counts: UVec3,
        _: u32,
    ) -> Vec4 {
        // NOTE: This debug mode visualises the number of lights within the cluster that contains
        // the fragment. It shows a sort of lighting complexity measure.
        let cluster_overlay_alpha = 0.1;
        let max_light_complexity_per_cluster = 64.0;

        output_color.x = (1.0 - cluster_overlay_alpha) * output_color.x
            + cluster_overlay_alpha
                * ((offset_and_counts.y + offset_and_counts.z) as f32)
                    .smooth_step(0.0, max_light_complexity_per_cluster);

        output_color.y = (1.0 - cluster_overlay_alpha) * output_color.y
            + cluster_overlay_alpha
                * (1.0
                    - ((offset_and_counts.y + offset_and_counts.z) as f32)
                        .smooth_step(0.0, max_light_complexity_per_cluster));

        output_color
    }
}

pub enum DebugClusterCoherency {}

impl ClusterDebugVisualization for DebugClusterCoherency {
    fn cluster_debug_visualization(
        _: &Lights,
        output_color: Vec4,
        _: f32,
        _: bool,
        _: UVec3,
        cluster_index: u32,
    ) -> Vec4 {
        // NOTE: Visualizes the cluster to which the fragment belongs
        let cluster_overlay_alpha = 0.1;
        let cluster_color = hsv2rgb(random_1d(cluster_index as f32), 1.0, 0.5);
        ((1.0 - cluster_overlay_alpha) * output_color.truncate()
            + cluster_overlay_alpha * cluster_color)
            .extend(output_color.w)
    }
}
