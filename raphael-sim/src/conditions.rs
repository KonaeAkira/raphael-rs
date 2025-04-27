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
    pub fn follow_up_condition(self) -> Option<Condition> {
        match self {
            Condition::Excellent => Some(Condition::Poor),
            Condition::GoodOmen => Some(Condition::Good),
            _ => None,
        }
    }

    pub fn follow_up_condition_after_steps(self, num_steps: u8) -> Option<Condition> {
        match num_steps {
            0 => Some(self),
            1 => self.follow_up_condition(),
            _ => None,
        }
    }
}
