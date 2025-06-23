use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub animation: AnimationConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnimationConfig {
    #[serde(default = "get_default_duration")]
    pub duration_milliseconds: u64,
    #[serde(default)]
    pub direction: AnimationDirection,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            duration_milliseconds: get_default_duration(),
            direction: AnimationDirection::default(),
        }
    }
}

#[derive(
    Debug, Default, PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum AnimationDirection {
    #[default]
    Out,
    In,
}

impl AnimationConfig {
    pub const DEFAULT_DURATION: Duration = Duration::from_secs(2);
}

const fn get_default_duration() -> u64 {
    AnimationConfig::DEFAULT_DURATION.as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config() {
        let config = Config::default();
        let config_string = toml::to_string_pretty(&config).unwrap();

        println!("{config_string}");
    }
}
