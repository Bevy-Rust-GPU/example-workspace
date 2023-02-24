use bevy_rust_gpu::prelude::{RustGpuEntryPoint, RustGpuEntryPointMappings, RustGpuEntryPointName};

pub enum MeshVertex {}

impl RustGpuEntryPoint for MeshVertex {
    const NAME: RustGpuEntryPointName = "mesh::entry_points::vertex";
    const MAPPINGS: RustGpuEntryPointMappings = &[
        (&[("VERTEX_TANGENTS", "tangent")], "none"),
        (&[("VERTEX_COLORS", "color")], "none"),
        (&[("SKINNED", "skinned")], "none"),
    ];
}

pub enum MeshFragment {}

impl RustGpuEntryPoint for MeshFragment {
    const NAME: RustGpuEntryPointName = "mesh::entry_points::fragment";
    const MAPPINGS: RustGpuEntryPointMappings = &[];
}

pub enum PbrFragment {}

impl RustGpuEntryPoint for PbrFragment {
    const NAME: RustGpuEntryPointName = "pbr::entry_points::fragment";
    const MAPPINGS: RustGpuEntryPointMappings = &[
        (&[("NO_TEXTURE_ARRAYS_SUPPORT", "texture")], "array"),
        (&[("NO_STORAGE_BUFFERS_SUPPORT", "uniform")], "storage"),
        (&[("VERTEX_POSITIONS", "position")], "none"),
        (&[("VERTEX_NORMALS", "normal")], "none"),
        (&[("VERTEX_UVS", "uv")], "none"),
        (&[("VERTEX_TANGENTS", "tangent")], "none"),
        (&[("VERTEX_COLORS", "color")], "none"),
        (&[("STANDARDMATERIAL_NORMAL_MAP", "normal_map")], "none"),
        (&[("SKINNED", "skinned")], "none"),
        (&[("TONEMAP_IN_SHADER", "tonemap")], "none"),
        (&[("DEBAND_DITHER", "deband")], "none"),
        (
            &[
                ("CLUSTERED_FORWARD_DEBUG_Z_SLICES", "debug_z_slices"),
                (
                    "CLUSTERED_FORWARD_DEBUG_CLUSTER_LIGHT_COMPLEXITY",
                    "debug_cluster_light_complexity",
                ),
                (
                    "CLUSTERED_FORWARD_DEBUG_CLUSTER_COHERENCY",
                    "debug_cluster_coherency",
                ),
            ],
            "none",
        ),
    ];
}
