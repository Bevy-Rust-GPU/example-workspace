use crate::prelude::{
    NormalMapTexture, STANDARD_MATERIAL_FLAGS_FLIP_NORMAL_MAP_Y,
    STANDARD_MATERIAL_FLAGS_TWO_COMPONENT_NORMAL_MAP,
};

use spirv_std::{
    glam::{Vec2, Vec3, Vec4},
    Sampler,
};

#[allow(unused_imports)]
use spirv_std::num_traits::Float;
pub trait BaseMaterialNormalMap {
    fn apply_flip_factor(_double_sided: bool, _is_front: bool, _normal: &mut Vec3) {}

    fn apply_pbr_input_n(
        _uv: Vec2,
        _tangent: Vec4,
        _standard_material_flags: u32,
        _normal_map_texture: &NormalMapTexture,
        _normal_map_sampler: &Sampler,
        _n: &mut Vec3,
    ) {
    }
}

pub enum StandardMaterialNormalMap {}

impl BaseMaterialNormalMap for StandardMaterialNormalMap {
    fn apply_flip_factor(double_sided: bool, is_front: bool, normal: &mut Vec3) {
        // NOTE: When NOT using normal-mapping, if looking at the back face of a double-sided
        // material, the normal needs to be inverted. This is a branchless version of that.
        *normal = (if !double_sided || is_front { 1.0 } else { 0.0 } * 2.0 - 1.0) * *normal;
    }

    fn apply_pbr_input_n(
        uv: Vec2,
        tangent: Vec4,
        standard_material_flags: u32,
        normal_map_texture: &NormalMapTexture,
        normal_map_sampler: &Sampler,
        n: &mut Vec3,
    ) {
        // NOTE: The mikktspace method of normal mapping explicitly requires that these NOT be
        // normalized nor any Gram-Schmidt applied to ensure the vertex normal is orthogonal to the
        // vertex tangent! Do not change this code unless you really know what you are doing.
        // http://www.mikktspace.com/
        let t: Vec3 = tangent.truncate();
        let b: Vec3 = tangent.w * n.cross(t);

        // Nt is the tangent-space normal.
        let mut nt = normal_map_texture
            .sample::<f32, Vec4>(*normal_map_sampler, uv)
            .truncate();
        if (standard_material_flags & STANDARD_MATERIAL_FLAGS_TWO_COMPONENT_NORMAL_MAP) != 0 {
            // Only use the xy components and derive z for 2-component normal maps.
            nt = (nt.truncate() * 2.0 - 1.0).extend(0.0);
            nt.z = (1.0 - nt.x * nt.x - nt.y * nt.y).sqrt();
        } else {
            nt = nt * 2.0 - 1.0;
        }
        // Normal maps authored for DirectX require flipping the y component
        if (standard_material_flags & STANDARD_MATERIAL_FLAGS_FLIP_NORMAL_MAP_Y) != 0 {
            nt.y = -nt.y;
        }
        // NOTE: The mikktspace method of normal mapping applies maps the tangent-space normal from
        // the normal map texture in this way to be an EXACT inverse of how the normal map baker
        // calculates the normal maps so there is no error introduced. Do not change this code
        // unless you really know what you are doing.
        // http://www.mikktspace.com/
        *n = nt.x * t + nt.y * b + nt.z * *n;
    }
}

impl BaseMaterialNormalMap for () {}
