use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub gravity: GravityConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GravityConfig {
    pub u0: f32,
    pub surfaces: HashMap<String, f32>,
}

pub fn load() -> Config {
    toml::from_str(include_str!("../../../config.toml")).expect("config.toml must parse")
}

impl Config {
    /// Per-surface ceiling; unknown surfaces get no pull rather than a default.
    pub fn ceiling(&self, surface: &str) -> f32 {
        self.gravity.surfaces.get(surface).copied().unwrap_or(0.0)
    }
}
