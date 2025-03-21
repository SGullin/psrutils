//! Proveds a RW interface for `.par` files.
//! 
//! # Examples
//! 
//! ```
//! # use psrutils::parfile::Parfile;
//! # use psrutils::parfile::FittedParameterValue;
//! # use psrutils::data_types::J2000Ra;
//! # fn test() -> Result<(), psrutils::error::PsruError> {
//! let par_text = "
//!     PSR    J0000-9999\n\
//!     RA     23:59:59.999\n\
//!     DEC    45:59:59.999\n\
//!     PEPOCH 55000\n\
//!     F0     9001 1 0.0001\n\
//!     DM     1001.1
//! ".as_bytes();
//! 
//! let par = Parfile::read(std::io::BufReader::new(par_text))?;
//! 
//! // This is the name
//! let name_par = &par.texts[0];
//! 
//! assert_eq!(name_par.name(), "PSR");
//! assert_eq!(name_par.value(), "J0000-9999");
//! 
//! // This is the right ascension
//! let ra = J2000Ra::new(23, 59, 59.999).unwrap();
//! let fpv = FittedParameterValue::JustValue(ra);
//! assert_eq!(&fpv, par.ra.value());
//! 
//! // This is the fundamental frequency
//! let f0_par = &par.parameters[1];
//! let fpv = FittedParameterValue::FitInfo{
//!     value: 9001.0,
//!     fit: true,
//!     error: 0.0001,
//! };
//! assert_eq!(f0_par.name(), "F0");
//! assert_eq!(f0_par.value(), &fpv);
//! 
//! # Ok(())
//! }
//! ```

use std::io::{BufRead, Write};

pub use glitch::Glitch;
pub use jump::Jump;
pub use parameters::{
    Parameter,
    FittedParameterValue, 
};
use parameters::{
    FittedParameter, 
    J2000Fit,
    parse_coord, 
    parse_count, 
    parse_fitted, 
    parse_flag, 
    parse_text, 
    COORDS
};

use crate::{data_types::{DECCoordType, RACoordType}, error::PsruError};

mod parameters;
mod glitch;
mod jump;
mod tests;

/// Time ephemeris used.
#[allow(missing_docs)]
#[derive(Debug, Default, PartialEq)]
pub enum TimeEphemeris {
    #[default]
    Unstated,

    IF99,
    FB90
}
/// Binary model used.
#[allow(missing_docs)]
#[derive(Debug, Default, PartialEq)]
pub enum BinaryModel {
    #[default]
    Unstated,

    BT,
    ELL1,
    DD,
    MSS,
}
/// T2C method used.
#[allow(missing_docs)]
#[derive(Debug, Default, PartialEq)]
pub enum T2CMethod {
    #[default]
    Unstated,

    IAU2000B,
    TEMPO
}
/// Error mode used.
#[allow(missing_docs)]
#[derive(Debug, Default, PartialEq)]
pub enum ErrorMode {
    #[default]
    Unstated,

    Mode0,
    Mode1,
}
/// Units used.
#[allow(missing_docs)]
#[derive(Debug, Default, PartialEq)]
pub enum Units {
    #[default]
    Unstated,

    SI,
    TCB,
    TDB,
}

/// Complete representation of a loaded .par file.
/// 
/// It follows the loose standards of TEMPO2, and as such is guaranteed to have
/// values for `PSR`, `F0`, `PEPOCH`, and `DM`. Some particular parameters 
/// (e.g. `units`) are _not_ given default values when absent from a loaded 
/// file, rather, they are set to `Unstated`. 
/// 
/// Glitches and jumps are stored in vectors. Since glitches are multi-line
/// parameters, they are kept track of with indices (e.g. `GLEP_1`) and 
/// disjunct ranges are considered erroneous.
/// 
/// All fields are public, since it is essentially just a datafile. There is,
/// however, a check of all values performed before writing. A failure in this
/// results in an error and no write.
#[derive(Debug, Default)]
pub struct Parfile {
    /// J2000 right ascension (hh:mm:ss.sss)
    pub ra: Parameter<J2000Fit<RACoordType>>,
    /// J2000 declination (dd:mm:ss.sss)
    pub dec: Parameter<J2000Fit<DECCoordType>>,

    /// All double precision parameters, and optional data on whether to fit 
    /// them, with errors. See `FittedParameter` for more info.
    pub parameters: Vec<FittedParameter>,
    /// All integer parameters.
    pub counts: Vec<Parameter<u32>>,
    /// All text parameters.
    pub texts: Vec<Parameter<String>>,
    /// All boolean flags.
    pub flags: Vec<Parameter<bool>>,

    /// Glitches, if any
    pub glitches: Vec<Glitch>,
    /// Jumps, if any
    pub jumps: Vec<Jump>,

    /// Which time ephemeris to use
    pub time_eph: TimeEphemeris, 
    /// Binary model
    pub binary_model: BinaryModel,
    /// Method for transforming from terrestrial to celestial frame 
    pub t2c_method: T2CMethod,

    /// What units to use.
    pub units: Units,
    /// Which error mode to use.
    pub error_mode: ErrorMode,
}

impl Parfile {
    /// Reads a stream as a .par file. Returns errors for malformed entries,
    /// duplicate entries, missing mandatory parameters, and some 
    /// out-of-bounds values.
    /// 
    /// Most parameters are `f64` values, but some are `String`, a couple are
    /// `u32`, and a few have their own enums to avoid excessive `String` 
    /// usage.
    pub fn read(reader: impl BufRead) -> Result<Self, PsruError> {
        let mut par = Parfile::default();

        for result in reader.lines() {
            let line = result.map_err(PsruError::IOError)?;
            if line.is_empty() { continue; }
            par.parse_line(&line)?;
        }

        par.check()?;

        Ok(par)
    }

    /// Writes itself to a stream. 
    /// 
    /// Note that the order of parameters and whitespace may differ from any
    /// input file used to construct it, but the contents will be consistent.
    pub fn write(&self, writer: &mut impl Write) -> Result<(), PsruError> {
        self.check()?;

        // It's nice to put the name up top, even though it is a regular text
        // parameter... so we extract it here.
        let name_index = self.texts
            .iter()
            .position(|t| t.name() == "PSR")
            .ok_or(PsruError::ParNoName)?;

        let mut texts = self.texts
            .iter()
            .collect::<Vec<_>>();
        
        let name = texts.remove(name_index);

        // The special fields
        for line in vec![
            format!("PSR {}\n", name.value()),
            format!("{}\n", self.ra),
            format!("{}\n", self.dec),
        ] {
            writer.write(line.as_bytes()).map_err(PsruError::IOError)?;
        }

        // Double params
        for parameter in &self.parameters {
            writer.write(parameter.to_string().as_bytes())
                .map_err(PsruError::IOError)?;
        }

        // Integer params
        for parameter in &self.counts {
            let line =  format!("{} {}\n", parameter.name(), parameter.value());
            writer.write(line.as_bytes()).map_err(PsruError::IOError)?;
        }

        // String params
        for parameter in texts {
            let line =  format!("{} {}\n", parameter.name(), parameter.value());
            writer.write(line.as_bytes()).map_err(PsruError::IOError)?;
        }

        // Flags
        for parameter in &self.flags {
            let line =  format!(
                "{} {}", 
                parameter.name(), 
                match parameter.value() {
                    true => "Y\n",
                    false => "N\n",
                }
            );
            writer.write(line.as_bytes()).map_err(PsruError::IOError)?;
        }

        // Oddballs
        if self.time_eph != TimeEphemeris::Unstated {
            let line = format!("TIMEEPH {:?}\n", self.time_eph);
            writer.write(line.as_bytes())
                .map_err(PsruError::IOError)?;
        }
        if self.binary_model != BinaryModel::Unstated {
            let line = format!("MODEL {:?}\n", self.binary_model);
            writer.write(line.as_bytes())
                .map_err(PsruError::IOError)?;
        }
        if self.units != Units::Unstated {
            let line = format!("UNITS {:?}\n", self.units);
            writer.write(line.as_bytes())
                .map_err(PsruError::IOError)?;
        }
        if self.t2c_method != T2CMethod::Unstated {
            let line = format!("T2CMETHOD {:?}\n", self.t2c_method);
            writer.write(line.as_bytes())
                .map_err(PsruError::IOError)?;
        }
        match self.error_mode {
            ErrorMode::Unstated => {},
            ErrorMode::Mode0 => {
                writer.write("MODE 0\n".as_bytes())
                    .map_err(PsruError::IOError)?;
            },
            ErrorMode::Mode1 => {
                writer.write("MODE 1\n".as_bytes())
                    .map_err(PsruError::IOError)?;
            },
        }

        // Glitches
        for glitch in &self.glitches {
            let lines = glitch.write();
            writer.write(lines.as_bytes()).map_err(PsruError::IOError)?;
        }

        // Jumps
        for jump in &self.jumps {
            let line = jump.write();
            writer.write(line.as_bytes()).map_err(PsruError::IOError)?;
        }

        Ok(())
    }

    fn parse_line(&mut self, line: &str) -> Result<(), PsruError> {
        let parts = line.split_whitespace().collect::<Vec<_>>();
        if parts.len() < 2 {
            return Err(PsruError::ParMissingValue(parts[0].to_string()));
        }

        if Glitch::parse(&parts, &mut self.glitches)? { return Ok(()); }
        if Jump::parse(&parts, &mut self.jumps)? { return Ok(()); }
        if self.parse_special(&parts)? { return Ok(()); }

        if let Some(flag) = parse_flag(&parts)? {
            self.flags.push(flag);
            return Ok(());
        }
        if let Some(param) = parse_fitted(&parts)? {
            self.parameters.push(param);
            return Ok(());
        }
        if let Some(param) = parse_count(&parts)? {
            self.counts.push(param);
            return Ok(());
        }
        if let Some(param) = parse_text(&parts)? {
            self.texts.push(param);
            return Ok(());
        }

        Ok(())
    }

    fn parse_special(
        &mut self, 
        parts: &[&str],
    ) -> Result<bool, PsruError> {
        let key = parts[0];
        let value = parts[1];

        // Coords
        if COORDS[0].1.contains(&key) {
            if *self.ra.value() != FittedParameterValue::Missing {
                return Err(PsruError::ParRepeatParam(COORDS[0].0.to_string()));
            }

            self.ra = Parameter::new(
                &COORDS[0],
                parse_coord::<RACoordType>(value, parts)?,
            );
            
            return Ok(true);
        }
        if COORDS[1].1.contains(&key) {
            if *self.dec.value() != FittedParameterValue::Missing { 
                return Err(PsruError::ParRepeatParam(COORDS[1].0.to_string()));
            }
            
            self.dec = Parameter::new(
                &COORDS[1],
                parse_coord::<DECCoordType>(value, parts)?,
            );
            
            return Ok(true);
        }

        // Which time ephemeris to use (IF99/FB90)
        if "TIMEEPH" == key {
            if self.time_eph != TimeEphemeris::Unstated {
                return Err(PsruError::ParRepeatParam(String::from("TIMEEPH")))
            }
            self.time_eph = match value {
                "IF99" => TimeEphemeris::IF99,
                "FB90" => TimeEphemeris::FB90,
                other => return Err(PsruError::UnknownTimeEphemeris(other.to_string()))
            };
            return Ok(true);
        }

        // Binary model
        if "MODEL" == key {
            if self.binary_model != BinaryModel::Unstated {
                return Err(PsruError::ParRepeatParam(String::from("MODEL")))
            }
            self.binary_model = match value {
                "BT" => BinaryModel::BT,
                "DD" => BinaryModel::DD,
                "ELL1" => BinaryModel::ELL1,
                "MSS" => BinaryModel::MSS,
                other => return Err(PsruError::UnknownBinaryModel(other.to_string()))
            };
            return Ok(true);
        }

        // Method for transforming from terrestrial to celestial frame 
        if "T2CMETHOD" == key {
            if self.t2c_method != T2CMethod::Unstated {
                return Err(PsruError::ParRepeatParam(String::from("T2CMETHOD")))
            }
            self.t2c_method = match value {
                "TEMPO" => T2CMethod::TEMPO,
                "IAU2000B" => T2CMethod::IAU2000B,
                other => return Err(PsruError::UnknownT2CMethod(other.to_string()))
            };
            return Ok(true);
        }

        // Units
        if "UNITS" == key {
            if self.units != Units::Unstated {
                return Err(PsruError::ParRepeatParam(String::from("UNITS")))
            }
            self.units = match value {
                "SI" => Units::SI,
                "TCB" => Units::TCB,
                "TDB" => Units::TDB,
                other => return Err(PsruError::UnknownUnits(other.to_string()))
            };
            return Ok(true);
        }

        if "MODE" == key {
            if self.error_mode != ErrorMode::Unstated {
                return Err(PsruError::ParRepeatParam(String::from("MODE")))
            }
            self.error_mode = match value {
                "0" => ErrorMode::Mode0,
                "1" => ErrorMode::Mode1,
                other => return Err(PsruError::UnknownErrorMode(other.to_string()))
            };
            return Ok(true);
        }

        Ok(false)
    }
    
    /// Performs a little check to see everything's ok.
    fn check(&self) -> Result<(), PsruError> {
        // Check mandatory params
        if self.texts
            .iter()
            .find(|t| t.name() == "PSR")
            .is_none() {
            return Err(PsruError::ParNoName);
        }
        self.parameters
            .iter()
            .find(|t| t.name() == "PEPOCH")
            .map(|p| 
                if match *p.value() {
                    FittedParameterValue::Missing => false,
                    FittedParameterValue::JustValue(v) => v > 0.0,
                    FittedParameterValue::FitInfo { value, .. } => value > 0.0,
                }{ Ok(()) } else { Err(PsruError::ParBadPEpoch) }
            )
            .unwrap_or(Err(PsruError::ParNoPEpoch))?;
        
        self.parameters
            .iter()
            .find(|t| t.name() == "F0")
            .map(|p| 
                if match *p.value() {
                    FittedParameterValue::Missing => false,
                    FittedParameterValue::JustValue(v) => v > 0.0,
                    FittedParameterValue::FitInfo { value, .. } => value > 0.0,
                } { Ok(()) } else { Err(PsruError::ParBadFrequency) }
            )
            .unwrap_or(Err(PsruError::ParNoFrequency))?;

        if self.parameters
            .iter()
            .find(|t| t.name() == "DM")
            .is_none() {
            return Err(PsruError::ParNoDispersion);
        }

        // Check for duplicates
        let p64dupes = find_duplicates(&self.parameters);
        if !p64dupes.is_empty() {
            return Err(PsruError::ParDuplicateParameters(
                p64dupes
                    .into_iter()
                    .map(|(i, j)| (
                        self.parameters[i].to_string(),
                        self.parameters[j].to_string(),
                    )).collect()
            ));
        }
        let ptdupes = find_duplicates(&self.texts);
        if !ptdupes.is_empty() {
            return Err(PsruError::ParDuplicateParameters(
                ptdupes
                    .into_iter()
                    .map(|(i, j)| (
                        self.parameters[i].to_string(),
                        self.parameters[j].to_string(),
                    )).collect()
            ));
        }
        let fdupes = find_duplicates(&self.flags);
        if !fdupes.is_empty() {
            return Err(PsruError::ParDuplicateParameters(
                fdupes
                    .into_iter()
                    .map(|(i, j)| (
                        self.parameters[i].name().to_string(),
                        self.parameters[j].name().to_string(),
                    )).collect()
            ));
        }

        // Check glitches
        for glitch in &self.glitches {
            glitch.check()?;
        }

        Ok(())
    }
}

fn find_duplicates<T>(params: &[Parameter<T>]) -> Vec<(usize, usize)> {
    params
        .iter()
        .enumerate()
        .filter_map(|(i, p1)| params[i+1..]
            .iter()
            .enumerate()
            .find(|(_, p2)| p1.name() == p2.name())
            .map(|(j, _)| (i, j))
        ).collect()
}
