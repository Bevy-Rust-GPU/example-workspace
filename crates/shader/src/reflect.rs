use spirv_std::glam::{Vec2, Vec3, Vec4};

pub trait Reflect {
    fn reflect(self, normal: Self) -> Self;
}

impl Reflect for Vec2 {
    fn reflect(self, normal: Self) -> Self {
        -2.0 * (self.dot(normal)) * normal + self
    }
}

impl Reflect for Vec3 {
    fn reflect(self, normal: Self) -> Self {
        -2.0 * (self.dot(normal)) * normal + self
    }
}

impl Reflect for Vec4 {
    fn reflect(self, normal: Self) -> Self {
        -2.0 * (self.dot(normal)) * normal + self
    }
}

