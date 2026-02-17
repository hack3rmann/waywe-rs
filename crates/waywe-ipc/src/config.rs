use glam::Vec2;
use rand::distr::{Distribution as _, Uniform};
use serde::{Deserialize, Serialize};
use std::{env, fs, io::ErrorKind, path::PathBuf, time::Duration};
use tracing::{error, info};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub animation: AnimationConfig,
    #[serde(default)]
    pub effects: Vec<Effects>,
}

impl Config {
    /// Tries to read config file from HOME paths. If fails, returns the default one.
    ///
    /// Waywe does not create the config file for you,
    /// but it looks for one in the following locations on UNIX systems:
    ///
    /// 1. `$XDG_CONFIG_HOME/waywe/config.toml`
    /// 2. `$HOME/.config/waywe/config.toml`
    /// 3. `/etc/waywe/config.toml`
    pub fn read() -> Self {
        const TRAILING: &str = "waywe/config.toml";

        let xdg_path = env::var_os("XDG_CONFIG_HOME").map(|xdg| {
            let mut p = PathBuf::from(xdg);
            p.push(TRAILING);
            p
        });

        let home_path = env::home_dir().map(|mut home| {
            home.push(".config");
            home.push(TRAILING);
            home
        });

        let etc_path = {
            let mut etc = PathBuf::from("/etc");
            etc.push(TRAILING);
            Some(etc)
        };

        let home_paths = [xdg_path, home_path, etc_path].into_iter().flatten();

        for path in home_paths {
            match fs::read_to_string(&path) {
                Ok(contents) => match toml::from_str(&contents) {
                    Ok(config) => {
                        info!("loaded config at {}", path.display());
                        return config;
                    }
                    Err(error) => {
                        error!(?error, "invalid config at {}", path.display());
                        continue;
                    }
                },
                Err(error) if error.kind() == ErrorKind::NotFound => {
                    continue;
                }
                Err(error) => {
                    error!(?error, "failed to read config at {}", path.display());
                    continue;
                }
            }
        }

        Config::default()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum Effects {
    Convolve(ConvolveConfig),
    Blur(BlurConfig),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConvolveConfig {
    pub kernel: Vec<f32>,
}

impl Default for ConvolveConfig {
    fn default() -> Self {
        #[rustfmt::skip]
        const SHARPEN: [f32; 9] = [
             0.0, -1.0,  0.0,
            -1.0,  5.0, -1.0,
             0.0, -1.0,  0.0,
        ];

        Self {
            kernel: SHARPEN.to_vec(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BlurConfig {
    pub n_levels: u32,
    pub level_multiplier: u32,
}

impl Default for BlurConfig {
    fn default() -> Self {
        Self {
            n_levels: 2,
            level_multiplier: 1,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
#[derive(Default)]
pub enum CenterPosition {
    Point {
        position: Vec2,
    },
    #[default]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "used for debugging only"]
    fn print_config() {
        let config = Config {
            animation: AnimationConfig {
                center_position: CenterPosition::Point {
                    position: Vec2::ZERO,
                },
                ..AnimationConfig::default()
            },
            effects: vec![],
        };
        let string = toml::to_string(&config).unwrap();
        println!("{string}");
    }
}
