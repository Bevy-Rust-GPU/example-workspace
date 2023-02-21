use spirv_std::glam::{Mat4, UVec3, UVec4, Vec3, Vec4};

use super::clustered_forward::CLUSTER_COUNT_SIZE;

#[derive(Default, Copy, Clone, PartialEq)]
#[repr(C)]
pub struct View {
    pub view_proj: Mat4,
    pub inverse_view_proj: Mat4,
    pub view: Mat4,
    pub inverse_view: Mat4,
    pub projection: Mat4,
    pub inverse_projection: Mat4,
    pub world_position: Vec3,
    // viewport(x_origin, y_origin, width, height)
    pub viewport: Vec4,
}

#[derive(Default, Copy, Clone, PartialEq)]
#[repr(C)]
pub struct PointLight {
    // For point lights: the lower-right 2x2 values of the projection matrix [2][2] [2][3] [3][2] [3][3]
    // For spot lights: the direction (x,z), spot_scale and spot_offset
    pub light_custom_data: Vec4,
    pub color_inverse_square_range: Vec4,
    pub position_radius: Vec4,
    // 'flags' is a bit field indicating various options. u32 is 32 bits so we have up to 32 options.
    pub flags: u32,
    pub shadow_depth_bias: f32,
    pub shadow_normal_bias: f32,
    pub spot_light_tan_angle: f32,
}

pub const POINT_LIGHT_FLAGS_SHADOWS_ENABLED_BIT: u32 = 1;
pub const POINT_LIGHT_FLAGS_SPOT_LIGHT_Y_NEGATIVE: u32 = 2;

#[derive(Default, Copy, Clone, PartialEq)]
#[repr(C)]
pub struct DirectionalLight {
    pub view_projection: Mat4,
    pub color: Vec4,
    pub direction_to_light: Vec3,
    // 'flags' is a bit field indicating various options. u32 is 32 bits so we have up to 32 options.
    pub flags: u32,
    pub shadow_depth_bias: f32,
    pub shadow_normal_bias: f32,
}

pub const DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT: u32 = 1;

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

#[cfg(feature = "NO_STORAGE_BUFFERS_SUPPORT")]
#[derive(Copy, Clone, PartialEq)]
#[repr(C)]
pub struct PointLights {
    pub data: [PointLight; 256],
}

#[cfg(feature = "NO_STORAGE_BUFFERS_SUPPORT")]
#[derive(Copy, Clone, PartialEq)]
#[repr(C)]
pub struct ClusterLightIndexLists {
    // each u32 contains 4 u8 indices into the PointLights array
    pub data: [UVec4; 1024],
}

impl ClusterLightIndexLists {
    pub fn get_light_id(&self, index: u32) -> u32 {
        #[cfg(feature = "NO_STORAGE_BUFFERS_SUPPORT")]
        {
            // The index is correct but in cluster_light_index_lists we pack 4 u8s into a u32
            // This means the index into cluster_light_index_lists is index / 4
            let v = self.data[(index >> 4) as usize];
            let indices = match ((index >> 2) & ((1 << 2) - 1)) as usize {
                0 => v.x,
                1 => v.y,
                2 => v.z,
                3 => v.w,
                _ => panic!(),
            };
            // And index % 4 gives the sub-index of the u8 within the u32 so we shift by 8 * sub-index
            (indices >> (8 * (index & ((1 << 2) - 1)))) & ((1 << 8) - 1)
        }

        #[cfg(not(feature = "NO_STORAGE_BUFFERS_SUPPORT"))]
        {
            unsafe { *self.data.index(index as usize) }
        }
    }
}

#[cfg(feature = "NO_STORAGE_BUFFERS_SUPPORT")]
#[derive(Clone, PartialEq)]
#[repr(C)]
pub struct ClusterOffsetsAndCounts {
    // each u32 contains a 24-bit index into ClusterLightIndexLists in the high 24 bits
    // and an 8-bit count of the number of lights in the low 8 bits
    pub data: [UVec4; 1024],
}

impl ClusterOffsetsAndCounts {
    pub fn unpack(&self, cluster_index: u32) -> UVec3 {
        #[cfg(feature = "NO_STORAGE_BUFFERS_SUPPORT")]
        {
            let v = self.data[(cluster_index >> 2) as usize];
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
            unsafe { self.data.index(cluster_index as usize) }.truncate()
        }
    }
}

#[cfg(not(feature = "NO_STORAGE_BUFFERS_SUPPORT"))]
#[repr(C)]
pub struct PointLights {
    pub data: spirv_std::RuntimeArray<PointLight>,
}

#[cfg(not(feature = "NO_STORAGE_BUFFERS_SUPPORT"))]
#[repr(C)]
pub struct ClusterLightIndexLists {
    pub data: RuntimeArray<u32>,
}

#[cfg(not(feature = "NO_STORAGE_BUFFERS_SUPPORT"))]
#[repr(C)]
pub struct ClusterOffsetsAndCounts {
    pub data: RuntimeArray<UVec4>,
}

#[repr(C)]
pub struct Globals {
    // The time since startup in seconds
    // Wraps to 0 after 1 hour.
    pub time: f32,
    // The delta time since the previous frame in seconds
    pub delta_time: f32,
    // Frame count since the start of the app.
    // It wraps to zero when it reaches the maximum value of a u32.
    pub frame_count: u32,

    #[cfg(feature = "SIXTEEN_BYTE_ALIGNMENT")]
    // WebGL2 structs must be 16 byte aligned.
    _wasm_padding: f32,
}
