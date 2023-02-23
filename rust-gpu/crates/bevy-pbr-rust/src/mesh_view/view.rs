use spirv_std::glam::{Mat4, Vec3, Vec4};

#[allow(unused_imports)]
use spirv_std::num_traits::Float;

use crate::prelude::mesh_position_local_to_world;

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

impl View {
    // NOTE: Correctly calculates the view vector depending on whether
    // the projection is orthographic or perspective.
    pub fn calculate_view(&self, world_position: Vec4, is_orthographic: bool) -> Vec3 {
        if is_orthographic {
            // Orthographic view vector
            Vec3::new(
                self.view_proj.x_axis.z,
                self.view_proj.y_axis.z,
                self.view_proj.z_axis.z,
            )
            .normalize()
        } else {
            // Only valid for a perpective projection
            (self.world_position - world_position.truncate()).normalize()
        }
    }

    pub fn mesh_position_world_to_clip(&self, world_position: Vec4) -> Vec4 {
        self.view_proj * world_position
    }

    // NOTE: The intermediate world_position assignment is important
    // for precision purposes when using the 'equals' depth comparison
    // function.
    pub fn mesh_position_local_to_clip(&self, model: Mat4, vertex_position: Vec4) -> Vec4 {
        let world_position = mesh_position_local_to_world(model, vertex_position);
        self.mesh_position_world_to_clip(world_position)
    }
}

