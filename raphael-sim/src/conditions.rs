#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Condition {
    Normal,
    Good,
    Excellent,
    Poor,
}

impl Condition {
    /// Condition of the next step when the current action advances a normal
    /// (non-expert) synthesis.
    ///
    /// Excellent is always followed by Poor. Good and Poor are always followed
    /// by Normal. The next condition after Normal is random, so Normal is used as
    /// the solver's existing deterministic approximation until it is observed.
    pub const fn next_after_step(self) -> Self {
        match self {
            Self::Excellent => Self::Poor,
            Self::Normal | Self::Good | Self::Poor => Self::Normal,
        }
    }
}
