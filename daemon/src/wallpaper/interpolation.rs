pub type Interpolation = fn(f32) -> f32;

pub fn ease_out(x: f32) -> f32 {
    1.0 - (x - 1.0).powi(2)
}

pub fn ease_in(x: f32) -> f32 {
    x.powi(2)
}

pub fn ease_in_out(x: f32) -> f32 {
    3.0 * x.powi(2) - 2.0 * x.powi(3)
}
