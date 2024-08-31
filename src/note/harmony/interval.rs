#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Interval(pub f32);
impl Eq for Interval {}
#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd for Interval {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}
impl Ord for Interval {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}

#[rustfmt::skip]
impl Interval {
    pub const ZERO:               Interval = Interval(0.0);

    pub const HALFSTEP:           Interval = Interval(1.0);
    pub const WHOLESTEP:          Interval = Interval(2.0);

    pub const UNISON:             Interval = Interval(0.0);
    pub const MINOR_SECOND:       Interval = Interval(1.0);
    pub const MAJOR_SECOND:       Interval = Interval(2.0);
    pub const MINOR_THIRD:        Interval = Interval(3.0);
    pub const MAJOR_THIRD:        Interval = Interval(4.0);
    pub const FOURTH:             Interval = Interval(5.0);
    pub const AUGMENTED_FOURTH:   Interval = Interval(6.0);
    pub const DIMINISHED_FIFTH:   Interval = Interval(6.0);
    pub const FIFTH:              Interval = Interval(7.0);
    pub const MINOR_SIXTH:        Interval = Interval(8.0);
    pub const MAJOR_SIXTH:        Interval = Interval(9.0);
    pub const MINOR_SEVENTH:      Interval = Interval(10.0);
    pub const MAJOR_SEVENTH:      Interval = Interval(11.0);
    pub const OCTAVE:             Interval = Interval(12.0);
    pub const MINOR_NINTH:        Interval = Interval(13.0);
    pub const MAJOR_NINTH:        Interval = Interval(14.0);
    pub const MINOR_TENTH:        Interval = Interval(15.0);
    pub const MAJOR_TENTH:        Interval = Interval(16.0);
    pub const ELEVENTH:           Interval = Interval(17.0);
    pub const AUGMENTED_ELEVENTH: Interval = Interval(18.0);
    pub const DIMINISHED_TWELFTH: Interval = Interval(18.0);
    pub const TWELFTH:            Interval = Interval(19.0);
    pub const MINOR_THIRTEENTH:   Interval = Interval(20.0);
    pub const MAJOR_THIRTEENTH:   Interval = Interval(21.0);
    pub const MINOR_FOURTEENTH:   Interval = Interval(22.0);
    pub const MAJOR_FOURTEENTH:   Interval = Interval(23.0);

    pub fn halfsteps(&self) -> f32 { self.0 }
    pub fn from_halfsteps(halfsteps: f32) -> Self { Interval(halfsteps) }
}

impl std::ops::Add<Interval> for Interval {
    type Output = Interval;
    fn add(self, rhs: Interval) -> Self::Output { Interval(self.0 + rhs.0) }
}
impl std::ops::Sub<Interval> for Interval {
    type Output = Interval;
    fn sub(self, rhs: Interval) -> Self::Output { Interval(self.0 - rhs.0) }
}
impl std::ops::Mul<f32> for Interval {
    type Output = Interval;
    fn mul(self, rhs: f32) -> Self::Output { Interval(self.0 * rhs) }
}
impl std::ops::Div<f32> for Interval {
    type Output = Interval;
    fn div(self, rhs: f32) -> Self::Output { Interval(self.0 / rhs) }
}
impl std::iter::Sum for Interval {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self { Interval(iter.map(|i| i.0).sum()) }
}
