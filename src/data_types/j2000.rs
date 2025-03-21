use std::{marker::PhantomData, str::FromStr};
use crate::{error::PsruError, parse_tools::parse_f64};

type Result<T> = std::result::Result<T, PsruError>;

/// Alias for [`J2000Coord<RACoordType>`].
pub type J2000Ra = J2000Coord<RACoordType>;
/// Alias for [`J2000Coord<DECCoordType>`].
pub type J2000Dec = J2000Coord<DECCoordType>;

/// Empty struct to define RA coords, see [`J2000Coord<RACoordType>`].
#[derive(Debug, Default, PartialEq)]
pub struct RACoordType;
/// Empty struct to define DEC coords, see [`J2000Coord<DECCoordType>`].
#[derive(Debug, Default, PartialEq)]
pub struct DECCoordType;

/// A J2000 coordinate. Comes in two variants, ra and dec, both using the same
/// underying data structure. 
#[derive(Debug, Default, PartialEq)]
pub struct J2000Coord<CT> {
    pub major: i8,
    pub minutes: u8,
    pub seconds: f64,

    _phantom: PhantomData<CT>,
}
impl<T> std::fmt::Display for J2000Coord<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}",
            self.major, self.minutes, self.seconds,
        )
    }
}
impl FromStr for J2000Coord<RACoordType> {
    type Err = PsruError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let coord_parts = s.split(":").collect::<Vec<_>>();
        if coord_parts.len() != 3 {
            return Err(PsruError::InvalidRA(s.to_string()));
        }
        
        let major = coord_parts[0]
            .parse::<i8>()
            .map_err(|_| PsruError::Unparsable { 
                value: coord_parts[0].to_string(), 
                to_type: "hours [0, 24)"
            })?;
        let minutes = coord_parts[1]
            .parse::<u8>()
            .map_err(|_| PsruError::Unparsable { 
                value: coord_parts[0].to_string(), 
                to_type: "minutes",
            })?;
        let seconds = parse_f64(coord_parts[2])?;

        let ra = Self { major, minutes, seconds, _phantom: PhantomData };
        ra.verify()?;

        Ok(ra)
    }
}
impl FromStr for J2000Coord<DECCoordType> {
    type Err = PsruError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let coord_parts = s.split(":").collect::<Vec<_>>();
        if coord_parts.len() != 3 {
            return Err(PsruError::InvalidRA(s.to_string()));
        }
        
        let major = coord_parts[0]
            .parse::<i8>()
            .map_err(|_| PsruError::Unparsable { 
                value: coord_parts[0].to_string(), 
                to_type: "degrees [-90, 90]"
            })?;
        let minutes = coord_parts[1]
            .parse::<u8>()
            .map_err(|_| PsruError::Unparsable { 
                value: coord_parts[0].to_string(), 
                to_type: "minutes",
            })?;
        let seconds = parse_f64(coord_parts[2])?;

        let dec = Self { major, minutes, seconds, _phantom: PhantomData };
        dec.verify()?;

        Ok(dec)
    }
}
impl J2000Coord<RACoordType> {
    pub fn new(hours: i8, minutes: u8, seconds: f64) -> Result<Self> {
        let ra = Self {
            major: hours,
            minutes,
            seconds,
            _phantom: PhantomData,
        };
        ra.verify()?;
        Ok(ra)
    }

    pub fn verify(&self) -> Result<()> {
        if self.major >= 24 
        || self.major < 0 
        || self.minutes >= 60 
        || self.seconds >= 60.0 {
            return Err(PsruError::InvalidRA(self.to_string()));
        }

        Ok(())
    }
}
impl J2000Coord<DECCoordType> {
    pub fn new(degrees: i8, minutes: u8, seconds: f64) -> Result<Self> {
        let dec = Self {
            major: degrees,
            minutes,
            seconds,
            _phantom: PhantomData,
        };
        dec.verify()?;
        Ok(dec)
    }

    pub fn verify(&self) -> Result<()> {
        if self.major < -90
        || self.major == -90 && (self.minutes > 0 || self.seconds > 0.0) 
        || self.major > 90 
        || self.major == 90 && (self.minutes > 0 || self.seconds > 0.0)
        || self.minutes >= 60 
        || self.seconds >= 60.0 {
            return Err(PsruError::InvalidDec(self.to_string()));
        }

        Ok(())
    }    
}
impl<T> J2000Coord<T> {
    /// Generates a single `f64` value in the same range as the original data.
    /// 
    /// # Examples
    /// ```
    /// # use psrutils::data_types::J2000Ra;
    /// let ra = J2000Ra::new(12, 30, 0.0).unwrap();
    /// let ra_f64 = ra.as_f64();
    /// assert_eq!(ra_f64, 12.5);
    /// ```
    pub fn as_f64(&self) -> f64 {
        self.major as f64
        + self.minutes as f64 / 60.0
        + self.seconds / 3600.0
    }
}
