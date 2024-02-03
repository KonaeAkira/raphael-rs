#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Progress {
    value: u32,
}

impl Progress {
    pub const fn scale(self, mul: u32, div: u32) -> Self {
        Self { value: self.value * mul / div }
    }

    pub const fn from_const(value: u32) -> Self {
        Self { value: value * 20 }
    }
}

impl std::convert::From<u32> for Progress {
    fn from(value: u32) -> Self {
        Self { value: value * 20 }
    }
}

impl std::convert::From<f32> for Progress {
    fn from(value: f32) -> Self {
        let converted = (value * 20.0) as u32;
        assert!(converted as f32 == value * 20.0);
        Self { value: converted }
    }
}

impl std::convert::Into<f32> for Progress {
    fn into(self) -> f32 {
        self.value as f32 / 20.0
    }
}

impl std::ops::Add for Progress {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            value: self.value + other.value,
        }
    }
}

impl std::ops::AddAssign for Progress {
    fn add_assign(&mut self, rhs: Self) {
        self.value += rhs.value;
    }
}

impl std::fmt::Display for Progress {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let result: f32 = (*self).into();
        write!(f, "{:.2}", result)
    }
}
