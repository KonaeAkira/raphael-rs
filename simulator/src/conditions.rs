#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Condition {
    Normal,
    Good,
    Excellent,
    Poor,
}
