use spirv_std::glam::{Vec2, Vec3, Vec4};

/// Rust implementation of WGSL saturate()
pub trait Saturate {
    fn saturate(self) -> Self;
}

impl Saturate for f32 {
    fn saturate(self) -> Self {
        self.clamp(0.0, 1.0)
    }
}

impl Saturate for Vec2 {
    fn saturate(self) -> Self {
        Vec2::new(self.x.saturate(), self.y.saturate())
    }
}

impl Saturate for Vec3 {
    fn saturate(self) -> Self {
        Vec3::new(self.x.saturate(), self.y.saturate(), self.z.saturate())
    }
}

impl Saturate for Vec4 {
    fn saturate(self) -> Self {
        Vec4::new(
            self.x.saturate(),
            self.y.saturate(),
            self.z.saturate(),
            self.w.saturate(),
        )
    }
}

