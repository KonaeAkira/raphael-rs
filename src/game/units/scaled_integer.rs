#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScaledU32<const C: u32> {
    value: u32,
}

impl<const C: u32> ScaledU32<C> {
    pub const fn new(value: u32) -> Self {
        Self { value: value * C }
    }

    pub const fn saturating_add(self, other: Self) -> Self {
        Self {
            value: self.value.saturating_add(other.value),
        }
    }

    pub const fn saturating_sub(self, other: Self) -> Self {
        Self {
            value: self.value.saturating_sub(other.value),
        }
    }

    pub const fn scale(self, mul: u32, div: u32) -> Self {
        Self {
            value: self.value * mul / div,
        }
    }
}

impl<const C: u32> std::convert::From<f32> for ScaledU32<C> {
    fn from(value: f32) -> Self {
        Self {
            value: (value * C as f32).ceil() as u32,
        }
    }
}

impl<const C: u32> std::convert::From<ScaledU32<C>> for f32 {
    fn from(value: ScaledU32<C>) -> Self {
        value.value as f32 / C as f32
    }
}
