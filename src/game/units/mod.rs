mod scaled_integer;

/// Used for progress values.
/// ```rust
/// use raphael::game::units::Progress;
/// Progress::new(100); // represents progress from 100 potency
/// ```
pub type Progress = scaled_integer::ScaledU32<20>;

/// Used for quality values.
/// ```rust
/// use raphael::game::units::Quality;
/// Quality::new(100); // represents quality from 100 potency
/// ```
pub type Quality = scaled_integer::ScaledU32<800>;

pub type CP = i16;
pub type Durability = i8;
