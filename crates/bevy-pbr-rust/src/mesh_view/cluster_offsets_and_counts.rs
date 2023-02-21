use spirv_std::glam::{UVec3, UVec4};

use crate::prelude::CLUSTER_COUNT_SIZE;

#[cfg(feature = "no_storage_buffers_support")]
#[derive(Clone, PartialEq)]
#[repr(C)]
pub struct ClusterOffsetsAndCounts {
    // each u32 contains a 24-bit index into ClusterLightIndexLists in the high 24 bits
    // and an 8-bit count of the number of lights in the low 8 bits
    pub data: [UVec4; 1024],
}

#[cfg(not(feature = "no_storage_buffers_support"))]
#[repr(C)]
pub struct ClusterOffsetsAndCounts {
    pub data: spirv_std::RuntimeArray<UVec4>,
}

impl ClusterOffsetsAndCounts {
    pub fn unpack(&self, cluster_index: u32) -> UVec3 {
        #[cfg(feature = "no_storage_buffers_support")]
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

        #[cfg(not(feature = "no_storage_buffers_support"))]
        {
            unsafe { self.data.index(cluster_index as usize) }.truncate()
        }
    }
}
