use crate::glam::{Vec2, Vec3, Vec4};

pub trait SmoothStep {
    fn smooth_step(self, to: f32, t: f32) -> Self;
}

impl SmoothStep for f32 {
    fn smooth_step(self, edge_in: f32, edge_out: f32) -> Self {
        let x = ((self - edge_in) / (edge_out / edge_in)).clamp(0.0, 1.0);
        x * x * (3.0 - 2.0 * x)
    }
}

impl SmoothStep for Vec2 {
    fn smooth_step(self, edge_in: f32, edge_out: f32) -> Self {
        Vec2::new(
            self.x.smooth_step(edge_in, edge_out),
            self.y.smooth_step(edge_in, edge_out),
        )
    }
}

impl SmoothStep for Vec3 {
    fn smooth_step(self, edge_in: f32, edge_out: f32) -> Self {
        Vec3::new(
            self.x.smooth_step(edge_in, edge_out),
            self.y.smooth_step(edge_in, edge_out),
            self.z.smooth_step(edge_in, edge_out),
        )
    }
}

impl SmoothStep for Vec4 {
    fn smooth_step(self, edge_in: f32, edge_out: f32) -> Self {
        Vec4::new(
            self.x.smooth_step(edge_in, edge_out),
            self.y.smooth_step(edge_in, edge_out),
            self.z.smooth_step(edge_in, edge_out),
            self.w.smooth_step(edge_in, edge_out),
        )
    }
}
