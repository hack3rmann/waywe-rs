use display_error_chain::ErrorChainExt;
use glam::Vec2;
use rand::distr::{Distribution as _, Uniform};
use serde::{Deserialize, Serialize};
use serde_dhall::StaticType;
use std::{env, path::PathBuf, time::Duration};
use tracing::{debug, error, info};

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, StaticType)]
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
        const TRAILING: &str = "waywe/config.dhall";

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
            if !path.exists() || !path.is_file() {
                continue;
            }

            match serde_dhall::from_file(&path).parse::<Config>() {
                Ok(config) => {
                    info!(path = %path.display(), "loaded config");
                    debug!("config {config:#?}");
                    return config;
                }
                Err(error) => {
                    error!(error = %error.chain(), path = %path.display(), "invalid config");
                    continue;
                }
            }
        }

        let default = Config::default();
        info!("loaded default config {default:#?}");

        default
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, StaticType)]
pub enum Effects {
    Convolve(ConvolveConfig),
    Blur(BlurConfig),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, StaticType)]
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

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, StaticType,
)]
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

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, StaticType)]
pub enum AnimationStyle {
    #[default]
    Circle,
    Slide,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, StaticType)]
#[serde(rename = "Transition")]
pub enum TransitionStyle {
    Circle {
        #[serde(rename = "center")]
        center_position: CenterPosition,
        direction: AnimationDirection,
    },
    Slide {
        angle: Angle,
    },
}

impl TransitionStyle {
    pub fn animation(&self) -> AnimationStyle {
        match self {
            Self::Circle { .. } => AnimationStyle::Circle,
            Self::Slide { .. } => AnimationStyle::Slide,
        }
    }
}

impl Default for TransitionStyle {
    fn default() -> Self {
        Self::Circle {
            center_position: CenterPosition::default(),
            direction: AnimationDirection::default(),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, StaticType)]
pub struct AnimationConfig {
    #[serde(default = "get_default_duration")]
    pub duration: u64,
    #[serde(default)]
    pub easing: Interpolation,
    #[serde(default)]
    pub style: TransitionStyle,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            duration: get_default_duration(),
            easing: Interpolation::default(),
            style: TransitionStyle::default(),
        }
    }
}

#[derive(
    Debug,
    Default,
    PartialEq,
    PartialOrd,
    Eq,
    Ord,
    Clone,
    Copy,
    Hash,
    Serialize,
    Deserialize,
    StaticType,
)]
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
    Clone, Copy, Debug, PartialEq, PartialOrd, Default, Serialize, Deserialize, StaticType,
)]
pub struct Bezier {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
}

#[derive(
    Clone, Copy, Debug, PartialEq, PartialOrd, Default, Serialize, Deserialize, StaticType,
)]
pub enum Interpolation {
    None,
    EaseIn,
    #[default]
    EaseOut,
    EaseInOut,
    Bezier(Bezier),
}

impl Interpolation {
    pub fn get(self, x: f32) -> f32 {
        match self {
            Interpolation::None => x,
            Interpolation::EaseIn => x * x,
            Interpolation::EaseOut => 1.0 - (1.0 - x) * (1.0 - x),
            Interpolation::EaseInOut => 3.0 * x * x - 2.0 * x * x * x,
            Interpolation::Bezier(Bezier { a, b, c, d }) => {
                Self::cubic_bezier(x, a.clamp(0.0, 1.0), b, c.clamp(0.0, 1.0), d)
            }
        }
    }

    pub fn cubic_bezier(x: f32, a: f32, b: f32, c: f32, d: f32) -> f32 {
        const N_ITERATIONS: usize = 10;
        const EPS: f32 = 1e-6;

        let mut t0 = 0.0;
        let mut t1 = 1.0;
        let mut t = x;

        for _ in 0..N_ITERATIONS {
            let x_t = Self::sample_curve(t, a, c);

            if (x_t - x).abs() < EPS {
                break;
            }

            if x_t < x {
                t0 = t;
            } else {
                t1 = t;
            }

            t = (t0 + t1) * 0.5;
        }

        Self::sample_curve(t, b, d)
    }

    fn sample_curve(t: f32, a: f32, b: f32) -> f32 {
        let u = 1.0 - t;

        3.0 * u * u * t * a + 3.0 * u * t * t * b + t * t * t
    }
}

#[derive(Clone, Default, Copy, Debug, PartialEq, Serialize, Deserialize, StaticType)]
pub enum CenterPosition {
    Point {
        x: f32,
        y: f32,
    },
    #[default]
    Random,
}

impl CenterPosition {
    pub fn get(self) -> Vec2 {
        match self {
            Self::Point { x, y } => Vec2::new(x, y),
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

#[derive(
    Clone, Default, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize, StaticType,
)]
pub enum Angle {
    Degrees(f32),
    #[default]
    Random,
}

impl Angle {
    pub fn get_degrees(self) -> f32 {
        match self {
            Self::Degrees(value) => value,
            Self::Random => {
                let distribution = Uniform::new_inclusive(0.0_f32, 360.0).unwrap();
                let mut rng = rand::rng();

                distribution.sample(&mut rng)
            }
        }
    }

    pub fn get_radians(self) -> f32 {
        self.get_degrees().to_radians()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "used for debugging only"]
    fn print_config_circle() {
        let config = Config {
            animation: AnimationConfig {
                style: TransitionStyle::default(),
                ..AnimationConfig::default()
            },
            effects: vec![],
        };
        let string = serde_dhall::serialize(&config)
            .static_type_annotation()
            .to_string()
            .unwrap();
        println!("{string}");
    }

    #[test]
    #[ignore = "used for debugging only"]
    fn print_config_slide() {
        let config = Config {
            animation: AnimationConfig {
                style: TransitionStyle::Slide {
                    angle: Angle::Random,
                },
                ..AnimationConfig::default()
            },
            effects: vec![],
        };
        let string = toml::to_string(&config).unwrap();
        println!("{string}");
    }
}
