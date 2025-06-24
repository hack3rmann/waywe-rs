use glam::Vec2;
use rand::distr::{Distribution as _, Uniform};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub animation: AnimationConfig,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AnimationConfig {
    #[serde(default = "get_default_duration")]
    pub duration_milliseconds: u64,
    #[serde(default)]
    pub direction: AnimationDirection,
    #[serde(default)]
    pub easing: Interpolation,
    #[serde(default)]
    pub center_position: CenterPosition,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            duration_milliseconds: get_default_duration(),
            direction: AnimationDirection::default(),
            easing: Interpolation::default(),
            center_position: CenterPosition::default(),
        }
    }
}

#[derive(
    Debug, Default, PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "kebab-case")]
pub enum AnimationDirection {
    Out,
    #[default]
    In,
}

impl AnimationConfig {
    pub const DEFAULT_DURATION: Duration = Duration::from_secs(2);
}

const fn get_default_duration() -> u64 {
    AnimationConfig::DEFAULT_DURATION.as_millis() as u64
}

#[derive(
    Clone, Copy, Debug, PartialEq, PartialOrd, Default, Eq, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "kebab-case")]
pub enum Interpolation {
    None,
    EaseIn,
    #[default]
    EaseOut,
    EaseInOut,
}

impl Interpolation {
    pub const fn get(self) -> InterpolationFn {
        match self {
            Interpolation::None => |x| x,
            Interpolation::EaseIn => |x| x * x,
            Interpolation::EaseOut => |x| 1.0 - (1.0 - x) * (1.0 - x),
            Interpolation::EaseInOut => |x| 3.0 * x * x - 2.0 * x * x * x,
        }
    }
}

pub type InterpolationFn = fn(f32) -> f32;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum CenterPosition {
    Point {
        position: Vec2,
    },
    Random,
}

impl CenterPosition {
    pub fn get(self) -> Vec2 {
        match self {
            Self::Point { position } => position,
            Self::Random => {
                let distribution = Uniform::new_inclusive(-1.0_f32, 1.0).unwrap();
                let mut rng = rand::rng();

                Vec2::new(
                    distribution.sample(&mut rng).powi(3),
                    distribution.sample(&mut rng).powi(3),
                )
            }
        }
    }
}

impl Default for CenterPosition {
    fn default() -> Self {
        Self::Random
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "used for debugging only"]
    fn print_config() {
        let config = Config {
            animation: AnimationConfig {
                center_position: CenterPosition::Point { position: Vec2::ZERO },
                ..AnimationConfig::default()
            }
        };
        let string = toml::to_string(&config).unwrap();
        println!("{string}");
    }
}
