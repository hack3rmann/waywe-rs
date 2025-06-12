use ffmpeg_sys_next::{AV_TIME_BASE_Q, AVRational};
use std::{
    fmt,
    num::{NonZeroI32, NonZeroI64},
    time::Duration,
};

/// 64-bit signed rational numbers, 32-bit for numerator, 32-bit for denominator
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct RatioI32 {
    pub numerator: i32,
    pub denominator: NonZeroI32,
}

impl RatioI32 {
    /// `RatioI32(0/1)` value
    pub const ZERO: Self = Self {
        numerator: 0,
        denominator: NonZeroI32::new(1).unwrap(),
    };

    /// `RatioI32(1/1)` value
    pub const ONE: Self = Self {
        numerator: 1,
        denominator: NonZeroI32::new(1).unwrap(),
    };

    /// Constructs new [`RatioI32`]
    ///
    /// # Note
    ///
    /// Returns [`None`] if `denominator` is `0`
    pub const fn new(numerator: i32, denominator: i32) -> Option<Self> {
        match NonZeroI32::new(denominator) {
            None => None,
            Some(non_zero) => Some(Self {
                numerator,
                denominator: non_zero,
            }),
        }
    }

    pub const fn from_backend(value: AVRational) -> Option<Self> {
        Self::new(value.num, value.den)
    }

    pub const fn to_backend(self) -> AVRational {
        AVRational {
            num: self.numerator,
            den: self.denominator.get(),
        }
    }

    pub const fn to_f32(self) -> f32 {
        self.numerator as f32 / self.denominator.get() as f32
    }

    pub const fn to_f64(self) -> f64 {
        self.numerator as f64 / self.denominator.get() as f64
    }

    /// `1 / x`
    pub const fn inv(self) -> Option<Self> {
        Self::new(self.denominator.get(), self.numerator)
    }

    pub const fn is_zero(self) -> bool {
        self.numerator == 0
    }

    /// Converts [`RatioI32`] to [`Duration`] assuming `self` contains number of seconds
    pub const fn to_duration_seconds(self) -> Duration {
        // HACK(hack3rmann): may overflow a lot
        let n_seconds = self.numerator as i64 / self.denominator.get() as i64;
        let n_nanoseconds =
            1_000_000_000_i64 * self.numerator as i64 / self.denominator.get() as i64;
        Duration::new(n_seconds.cast_unsigned(), n_nanoseconds as u32)
    }
}

impl Default for RatioI32 {
    fn default() -> Self {
        Self::ZERO
    }
}

impl fmt::Debug for RatioI32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.numerator, self.denominator)
    }
}

/// Frame duration with chosen time base
#[derive(Clone, Copy, Debug)]
pub struct FrameDuration {
    /// Time units in seconds
    pub base: RatioI32,
    /// Duration in `base` units
    pub duration: NonZeroI64,
}

impl FrameDuration {
    /// Fallback time base
    pub const FALLBACK_BASE: RatioI32 = RatioI32::from_backend(AV_TIME_BASE_Q).unwrap();

    /// Converts [`FrameDuration`] to just [`Duration`]
    pub const fn to_duration(self) -> Duration {
        // HACK(hack3rmann): may overflow a lot
        let n_seconds =
            self.duration.get() * self.base.numerator as i64 / self.base.denominator.get() as i64;
        let n_nanoseconds = 1_000_000_000_i64 * self.duration.get() * self.base.numerator as i64
            / self.base.denominator.get() as i64;
        Duration::new(n_seconds as u64, n_nanoseconds as u32)
    }
}
