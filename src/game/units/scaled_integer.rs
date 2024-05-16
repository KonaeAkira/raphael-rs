use core::fmt;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ScaledU32<const C: u32> {
    inner_value: u32,
}

impl<const C: u32> ScaledU32<C> {
    pub const fn new(value: u32) -> Self {
        Self {
            inner_value: value * C,
        }
    }

    pub const fn add(self, other: Self) -> Self {
        Self {
            inner_value: self.inner_value + other.inner_value,
        }
    }

    pub const fn sub(self, other: Self) -> Self {
        Self {
            inner_value: self.inner_value - other.inner_value,
        }
    }

    pub const fn saturating_sub(self, other: Self) -> Self {
        Self {
            inner_value: self.inner_value.saturating_sub(other.inner_value),
        }
    }

    pub const fn scale(self, mul: u32, div: u32) -> Self {
        #[cfg(test)]
        assert!(self.inner_value * mul % div == 0);
        Self {
            inner_value: self.inner_value * mul / div,
        }
    }
}

impl<const C: u32> std::convert::From<f32> for ScaledU32<C> {
    fn from(value: f32) -> Self {
        Self {
            inner_value: (value * C as f32).ceil() as u32,
        }
    }
}

impl<const C: u32> std::convert::From<ScaledU32<C>> for f32 {
    fn from(value: ScaledU32<C>) -> Self {
        value.inner_value as f32 / C as f32
    }
}

impl<const C: u32> std::convert::From<f64> for ScaledU32<C> {
    fn from(value: f64) -> Self {
        Self {
            inner_value: (value * C as f64).ceil() as u32,
        }
    }
}

impl<const C: u32> std::convert::From<ScaledU32<C>> for f64 {
    fn from(value: ScaledU32<C>) -> Self {
        value.inner_value as f64 / C as f64
    }
}

impl<const C: u32> fmt::Debug for ScaledU32<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScaledU32")
            .field("value", &(self.inner_value as f32 / C as f32))
            .field("inner_value", &self.inner_value)
            .finish()
    }
}
