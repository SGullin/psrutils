use std::str::FromStr;

use crate::error::PsruError;

/// Represents a date-time in MJD.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Mjd {
    int: u32,
    frac: f64,
}

impl std::fmt::Display for Mjd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MJD {}.{}", self.int, &self.frac.to_string()[1..])
    }
}
impl FromStr for Mjd {
    type Err = PsruError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err = |s: &str| PsruError::Unparsable { 
            value: s.to_string(), 
            to_type: "MJD" 
        };

        let parts = s.split('.').collect::<Vec<_>>();
        if parts.len() == 1 {
            Ok(Self {
                int: parts[0].parse().map_err(|_| err(s))?,
                frac: 0.0,
            })
        }
        else if parts.len() == 2 {
            Ok(Self {
                int: parts[0].parse().map_err(|_| err(s))?,
                frac: parts[1].parse().map_err(|_| err(s))?,
            })
        }
        else {
            Err(err(s))
        }
    }
}
impl Mjd {
    /// # Panics
    /// `panic`s if the fractional part is outside of the range [0, 1).
    pub fn new(int: u32, frac: f64) -> Self {
        assert!(frac < 1.0);
        assert!(frac >= 0.0);

        Self {
            int,
            frac
        }
    }

    /// Converts the value into a pure `f64`.
    pub const fn to_f64(&self) -> f64 {
        self.int as f64 + self.frac
    }
}
