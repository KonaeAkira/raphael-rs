#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Condition {
    Normal,
    Good,
    Excellent,
    Poor,
    // Next Condition is Good
    GoodOmen,
    // Half Durability Loss
    Sturdy,
    // Increase Success of RNG Skills by 25%
    Centered,
    // CP Cost reduced by 50%
    Pliant,
    // Buff lasts 2 steps longer
    Primed,
    // Progress + 50%
    Malleable,
}

impl Condition {
    pub fn follow_up_condition(self) -> Condition {
        match self {
            Condition::Excellent => Condition::Poor,
            Condition::GoodOmen => Condition::Good,
            _ => Condition::Normal,
        }
    }

    pub const fn into_bits(self) -> u8 {
        match self {
            Self::Normal => 0,
            Self::Good => 1,
            Self::Excellent => 2,
            Self::Poor => 3,
            Self::GoodOmen => 5,
            Self::Sturdy => 6,
            Self::Centered => 7,
            Self::Pliant => 8,
            Self::Primed => 9,
            Self::Malleable => 10,
        }
    }

    pub const fn from_bits(value: u8) -> Self {
        match value {
            0 => Self::Normal,
            1 => Self::Good,
            2 => Self::Excellent,
            3 => Self::Poor,
            5 => Self::GoodOmen,
            6 => Self::Sturdy,
            7 => Self::Centered,
            8 => Self::Pliant,
            9 => Self::Primed,
            10 => Self::Malleable,
            _ => Self::Normal,
        }
    }
}
