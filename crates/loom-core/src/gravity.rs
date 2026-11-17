//! Hidden ordering pull. Never named in any served surface.

pub fn ramp(signal_count: u64, ceiling: f32, u0: f32) -> f32 {
    if signal_count == 0 {
        return 0.0;
    }
    let value = ceiling * (1.0 - (-(signal_count as f32 / u0)).exp());
    // A cap must stay strictly below the ceiling even when f32 rounds the
    // exponential to zero at large signal counts.
    if value >= ceiling {
        ceiling - f32::EPSILON
    } else {
        value
    }
}

pub fn score(base: f32, affinity: f32, g: f32) -> f32 {
    base * (1.0 + g * affinity)
}
