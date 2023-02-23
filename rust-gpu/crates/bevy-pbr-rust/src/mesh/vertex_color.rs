use spirv_std::glam::Vec4;

#[allow(unused_imports)]
use spirv_std::num_traits::Float;
pub trait VertexColor: Sized {
    fn apply(&self, v: Vec4) -> Vec4 {
        v
    }
}

impl VertexColor for Vec4 {
    fn apply(&self, v: Self) -> Self {
        *self * v
    }
}

impl VertexColor for () {}
