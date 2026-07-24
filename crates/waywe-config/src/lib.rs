use glam::Vec2;
use miette::Diagnostic;
use rand::distr::{Distribution as _, Uniform};
use serde::{Deserialize, Serialize};
use serde_dhall::StaticType;
use smallvec::SmallVec;
use static_assertions::assert_impl_all;
use std::{
    borrow::Borrow,
    env,
    path::{Path, PathBuf},
    time::Duration,
};
use thiserror::Error;

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, StaticType)]
pub struct Config {
    pub animation: AnimationConfig,
    #[serde(default)]
    pub effects: Vec<Effects>,
    #[serde(default)]
    pub config: ConfigOpts,
}
assert_impl_all!(Config: Send, Sync);

impl Config {
    pub fn config_paths() -> SmallVec<[PathBuf; 3]> {
        const WAYWE: &str = "waywe";

        let xdg_path = env::var_os("XDG_CONFIG_HOME").map(|xdg| {
            let mut path = PathBuf::from(xdg);
            path.push(WAYWE);
            path
        });

        let mut home_path = env::home_dir().map(|mut home| {
            home.extend([".config", WAYWE]);
            home
        });

        if xdg_path == home_path {
            home_path = None;
        }

        let etc_path = Some(PathBuf::from_iter(["/etc", WAYWE]));

        [xdg_path, home_path, etc_path]
            .into_iter()
            .flatten()
            .collect()
    }

    pub fn read_from(path: Option<impl AsRef<Path>>) -> Result<Self, ReadConfigError> {
        match path {
            Some(path) => Self::read(path),
            None => Self::read_from_config_paths(),
        }
    }

    pub fn read(path: impl AsRef<Path>) -> Result<Self, ReadConfigError> {
        let path = path.as_ref();

        serde_dhall::from_file(path)
            .parse::<Self>()
            .map_err(|error| DhallErrorSource {
                path: path.to_owned(),
                error,
            })
            .map_err(|error| ReadConfigError::InvalidConfig(Box::new(error)))
    }

    /// Tries to read config file from HOME paths. If fails, returns the default one.
    ///
    /// Waywe does not create the config file for you,
    /// but it looks for one in the following locations on UNIX systems:
    ///
    /// 1. `$XDG_CONFIG_HOME/waywe/config.dhall`
    /// 2. `$HOME/.config/waywe/config.dhall`
    /// 3. `/etc/waywe/config.dhall`
    pub fn read_from_config_paths() -> Result<Self, ReadConfigError> {
        let mut errors = vec![];

        for mut path in Self::config_paths() {
            path.push("config.dhall");

            match serde_dhall::from_file(&path).parse::<Self>() {
                Ok(config) => return Ok(config),
                Err(error) => errors.push(DhallErrorSource { path, error }),
            }
        }

        Err(
            if let Some(actual_config_error_index) = errors.iter_mut().position(|source| {
                !matches!(
                    source.error,
                    serde_dhall::Error::Dhall(dhall::error::Error::Io(_))
                )
            }) {
                let actual_error = errors.swap_remove(actual_config_error_index);
                ReadConfigError::InvalidConfig(Box::new(actual_error))
            } else {
                ReadConfigError::ConfigUnreachable { related: errors }
            },
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy, PartialEq, Hash, StaticType)]
pub struct ConfigOpts {
    pub disable_hot_reload: bool,
}

#[derive(Debug, Error, Diagnostic)]
#[diagnostic(code(waywe::dhall::error))]
#[error("failed to run Dhall '{path}'")]
pub struct DhallErrorSource {
    pub path: PathBuf,
    #[source]
    pub error: serde_dhall::Error,
}

impl Borrow<dyn Diagnostic> for Box<DhallErrorSource> {
    fn borrow(&self) -> &(dyn Diagnostic + 'static) {
        &**self
    }
}

#[derive(Debug, Error, Diagnostic)]
pub enum ReadConfigError {
    #[error("all config search paths lead to invalid config")]
    #[diagnostic(
        code(waywe::config::invalid),
        help("check config validity with `waywe config validate` and check file permissions")
    )]
    ConfigUnreachable {
        #[related]
        related: Vec<DhallErrorSource>,
    },
    #[error("invalid config")]
    #[diagnostic(
        code(waywe::config::invalid),
        help("check config validity with `waywe config validate`")
    )]
    InvalidConfig(
        #[from]
        #[diagnostic_source]
        Box<DhallErrorSource>,
    ),
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
pub enum Transition {
    Circle {
        center: CenterPosition,
        direction: AnimationDirection,
    },
    Slide {
        angle: Angle,
    },
}

impl Transition {
    pub fn animation(&self) -> AnimationStyle {
        match self {
            Self::Circle { .. } => AnimationStyle::Circle,
            Self::Slide { .. } => AnimationStyle::Slide,
        }
    }
}

impl Default for Transition {
    fn default() -> Self {
        Self::Circle {
            center: CenterPosition::default(),
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
    pub style: Transition,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            duration: get_default_duration(),
            easing: Interpolation::default(),
            style: Transition::default(),
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
                style: Transition::default(),
                ..AnimationConfig::default()
            },
            effects: vec![],
            config: ConfigOpts {
                disable_hot_reload: true,
            },
        };
        let string = serde_dhall::serialize(&config)
            .static_type_annotation()
            .to_string()
            .unwrap();
        println!("{string}");
    }
}
