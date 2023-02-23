use spirv_std::glam::UVec4;

pub trait ClusterLightIndexLists {
    fn get_light_id(&self, index: u32) -> u32;
}

#[derive(Copy, Clone, PartialEq)]
#[repr(C)]
pub struct ClusterLightIndexListsUniform {
    // each u32 contains 4 u8 indices into the PointLights array
    pub data: [UVec4; 1024],
}

impl ClusterLightIndexLists for ClusterLightIndexListsUniform {
    fn get_light_id(&self, index: u32) -> u32 {
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
}

#[repr(C)]
pub struct ClusterLightIndexListsStorage {
    pub data: spirv_std::RuntimeArray<u32>,
}

impl ClusterLightIndexLists for ClusterLightIndexListsStorage {
    fn get_light_id(&self, index: u32) -> u32 {
        unsafe { *self.data.index(index as usize) }
    }
}

