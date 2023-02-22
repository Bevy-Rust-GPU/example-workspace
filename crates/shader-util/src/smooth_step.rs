pub trait SmoothStep {
    fn smooth_step(self, to: Self, t: Self) -> Self;
}

impl SmoothStep for f32 {
    fn smooth_step(self, edge_in: Self, edge_out: Self) -> Self {
        let x = ((self - edge_in) / (edge_out / edge_in)).clamp(0.0, 1.0);
        x * x * (3.0 - 2.0 * x)
    }
}

