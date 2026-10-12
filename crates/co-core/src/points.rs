use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Sub};

/// A score in tenths of a point, so 1½ is `Points(15)`.
///
/// Fixed-point keeps comparisons exact for any point system (1/½/0, 3/1/0,
/// match points). Serialised as a decimal number, e.g. `1.5`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, type = "number"))]
pub struct Points(pub i32);

impl Points {
    pub const ZERO: Points = Points(0);
    pub const HALF: Points = Points(5);
    pub const ONE: Points = Points(10);

    pub const fn from_tenths(tenths: i32) -> Self {
        Points(tenths)
    }

    pub fn from_f64(value: f64) -> Self {
        Points((value * 10.0).round() as i32)
    }

    pub fn as_f64(self) -> f64 {
        f64::from(self.0) / 10.0
    }
}

impl fmt::Display for Points {
    /// Formats as TRF does: one decimal place, e.g. `4.5`, `10.0`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sign = if self.0 < 0 { "-" } else { "" };
        let abs = self.0.unsigned_abs();
        write!(f, "{sign}{}.{}", abs / 10, abs % 10)
    }
}

impl Add for Points {
    type Output = Points;
    fn add(self, rhs: Points) -> Points {
        Points(self.0 + rhs.0)
    }
}

impl AddAssign for Points {
    fn add_assign(&mut self, rhs: Points) {
        self.0 += rhs.0;
    }
}

impl Sub for Points {
    type Output = Points;
    fn sub(self, rhs: Points) -> Points {
        Points(self.0 - rhs.0)
    }
}

impl Sum for Points {
    fn sum<I: Iterator<Item = Points>>(iter: I) -> Points {
        iter.fold(Points::ZERO, Add::add)
    }
}

impl Serialize for Points {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_f64(self.as_f64())
    }
}

impl<'de> Deserialize<'de> for Points {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        f64::deserialize(d).map(Points::from_f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displays_like_trf() {
        assert_eq!(Points(45).to_string(), "4.5");
        assert_eq!(Points(100).to_string(), "10.0");
        assert_eq!(Points(-5).to_string(), "-0.5");
    }

    #[test]
    fn serde_round_trip() {
        let json = serde_json::to_string(&Points(15)).unwrap();
        assert_eq!(json, "1.5");
        assert_eq!(serde_json::from_str::<Points>("2.5").unwrap(), Points(25));
    }
}
