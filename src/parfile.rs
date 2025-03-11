use std::io::{BufRead, Write};

use glitch::Glitch;
use jump::Jump;
use error::ParParseError;

mod glitch;
mod jump;
mod error;
mod tests;

#[derive(Debug)]
pub enum FittedParameter {
    JustValue(f64),
    FitInfo {
        value: f64,
        fit: bool,
        error: f64,
    }    
}

#[derive(Debug, Default, PartialEq)]
pub enum FittedCoord {
    #[default]
    Missing,

    JustValue(i8, u8, f64),
    FitInfo {
        value: (i8, u8, f64),
        fit: bool,
        error: f64,
    }    
}

#[derive(Debug, Default)]
pub struct Parameter<T> {
    name: &'static str,
    description: &'static str,
    value: T,
}
impl<T> Parameter<T> {
    pub fn name(&self) -> &str { self.name }
    pub fn description(&self) -> &str { self.description }
    pub fn value(&self) -> &T { &self.value }
}
impl std::fmt::Display for Parameter<FittedParameter> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.value {
            FittedParameter::FitInfo { value, fit, error } => write!(
                f,
                "{} {} {} {}\n",
                self.name,
                value,
                if fit {"1"} else {"0"},
                error,
            ),
            FittedParameter::JustValue(v) => write!(f, "{} {}\n", self.name, v),
        }
    }
}
impl std::fmt::Display for Parameter<FittedCoord> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.value {
            FittedCoord::Missing => write!(f, "MISSING PARAMETER"),
            FittedCoord::FitInfo { value, fit, error } => write!(
                f,
                "{} {}:{}:{} {} {}\n",
                self.name,
                value.0, value.1, value.2,
                if fit {"1"} else {"0"},
                error,
            ),
            FittedCoord::JustValue(v1, v2, v3) 
                => write!(f, "{} {}:{}:{}\n", self.name, v1, v2, v3),
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub enum TimeEphemeris {
    #[default]
    Unstated,

    IF99,
    FB90
}
#[derive(Debug, Default, PartialEq)]
pub enum BinaryModel {
    #[default]
    Unstated,

    BT,
    ELL1,
    DD,
    MSS,
}
#[derive(Debug, Default, PartialEq)]
pub enum T2CMethod {
    #[default]
    Unstated,

    IAU2000B,
    TEMPO
}
#[derive(Debug, Default, PartialEq)]
pub enum ErrorMode {
    #[default]
    Unstated,

    Mode0,
    Mode1,
}
#[derive(Debug, Default, PartialEq)]
pub enum Units {
    #[default]
    Unstated,

    SI,
    TCD,
    TDB,
}

/// Complete representation of a loaded .par file.
/// 
/// It follows the loose standards of TEMPO2, and as such is guaranteed to have
/// values for `PSR`, `RA`, `DEC`, `F0`, `PEPOCH`, and `DM`. Some particular
/// parameters (e.g. `units`) are _not_ given default values when absent from
/// a loaded file, rather, they are set to `Unstated`. 
/// 
/// Glitches and jumps are stored in vectors. Since glitches are multi-line
/// parameters, they are kept track of with indices (e.g. `GLEP_1`) and 
/// disjunct ranges are considered erroneous.
#[derive(Debug, Default)]
pub struct Parfile {
    /// J2000 right ascension (hh:mm:ss.sss)
    ra: Parameter<FittedCoord>,
    /// J2000 declination (dd:mm:ss.sss)
    dec: Parameter<FittedCoord>,

    parameters: Vec<Parameter<FittedParameter>>,
    counts: Vec<Parameter<u32>>,
    texts: Vec<Parameter<String>>,
    flags: Vec<Parameter<()>>,

    /// Glitches, if any
    glitches: Vec<Glitch>,
    /// Jumps, if any
    jumps: Vec<Jump>,

    /// Which time ephemeris to use
    time_eph: TimeEphemeris, 
    /// Binary model
    binary_model: BinaryModel,
    /// Method for transforming from terrestrial to celestial frame 
    t2c_method: T2CMethod,

    units: Units,
    error_mode: ErrorMode,
}

impl Parfile {
    /// Reads a stream as a .par file. Returns errors for malformed entries,
    /// duplicate entries, missing mandatory parameters, and some 
    /// out-of-bounds values.
    /// 
    /// Most parameters are `f64` values, but some are `String`, a couple are
    /// `u32`, and a few have their own enums to avoid excessive `String` 
    /// usage.
    pub fn read(reader: impl BufRead) -> Result<Self, ParParseError> {
        let mut par = Parfile::default();

        for result in reader.lines() {
            let line = result.map_err(ParParseError::IOError)?;
            if line.is_empty() { continue; }
            par.parse_line(&line)?;
        }

        par.check()?;

        Ok(par)
    }

    /// Writes itself to a stream. 
    /// 
    /// Note that the order of parameters and whitespace may differ from any
    /// input file used to construct it, but will be consistent.
    pub fn write(&self, writer: &mut impl Write) -> Result<(), ParParseError> {
        // It's nice to put the name up top, even though it is a regular text
        // parameter... so we extract it here.
        let name_index = self.texts
            .iter()
            .position(|t| t.name == "PSR")
            .ok_or(ParParseError::NoName)?;

        let mut texts = self.texts
            .iter()
            .collect::<Vec<_>>();
        
        let name = texts.remove(name_index);

        // The special fields
        for line in vec![
            format!("PSR {}\n", name.value),
            format!("{}\n", self.ra),
            format!("{}\n", self.dec),
        ] {
            writer.write(line.as_bytes()).map_err(ParParseError::IOError)?;
        }

        // Double params
        for parameter in &self.parameters {
            writer.write(parameter.to_string().as_bytes())
                .map_err(ParParseError::IOError)?;
        }

        // Integer params
        for parameter in &self.counts {
            let line =  format!("{} {}\n", parameter.name, parameter.value);
            writer.write(line.as_bytes()).map_err(ParParseError::IOError)?;
        }

        // String params
        for parameter in texts {
            let line =  format!("{} {}\n", parameter.name, parameter.value);
            writer.write(line.as_bytes()).map_err(ParParseError::IOError)?;
        }

        // Flags
        for parameter in &self.flags {
            let line =  format!("{}\n", parameter.name);
            writer.write(line.as_bytes()).map_err(ParParseError::IOError)?;
        }

        // Oddballs
        if self.time_eph != TimeEphemeris::Unstated {
            let line = format!("TIMEEPH {:?}\n", self.time_eph);
            writer.write(line.as_bytes())
                .map_err(ParParseError::IOError)?;
        }
        if self.binary_model != BinaryModel::Unstated {
            let line = format!("MODEL {:?}\n", self.binary_model);
            writer.write(line.as_bytes())
                .map_err(ParParseError::IOError)?;
        }
        if self.units != Units::Unstated {
            let line = format!("UNITS {:?}\n", self.units);
            writer.write(line.as_bytes())
                .map_err(ParParseError::IOError)?;
        }
        if self.t2c_method != T2CMethod::Unstated {
            let line = format!("T2CMETHOD {:?}\n", self.t2c_method);
            writer.write(line.as_bytes())
                .map_err(ParParseError::IOError)?;
        }
        match self.error_mode {
            ErrorMode::Unstated => {},
            ErrorMode::Mode0 => {
                writer.write("MODE 0\n".as_bytes())
                    .map_err(ParParseError::IOError)?;
            },
            ErrorMode::Mode1 => {
                writer.write("MODE 1\n".as_bytes())
                    .map_err(ParParseError::IOError)?;
            },
        }

        // Glitches
        for glitch in &self.glitches {
            let lines = glitch.write();
            writer.write(lines.as_bytes()).map_err(ParParseError::IOError)?;
        }

        // Jumps
        for jump in &self.jumps {
            let line = jump.write();
            writer.write(line.as_bytes()).map_err(ParParseError::IOError)?;
        }

        Ok(())
    }

    fn parse_line(&mut self, line: &str) -> Result<(), ParParseError> {
        let parts = line.split_whitespace().collect::<Vec<_>>();

        if parts.len() == 1 {
            // Must be a flag
            self.flags.push(parse_flag(&parts[0])?);
            return Ok(());
        }

        if Glitch::parse(&parts, &mut self.glitches)? { return Ok(()); }
        if Jump::parse(&parts, &mut self.jumps)? { return Ok(()); }
        if self.parse_special(&parts)? { return Ok(()); }

        if let Some(param) = parse_param64(&parts)? {
            // Must be a fit value
            self.parameters.push(param);
            return Ok(());
        }

        // Else must be either u32 or text parameter
        let key = parts[0];
        let value = parts[1];
        let p32 = PARAMETERS_U32
            .iter()
            .find(|p| p.0 == key || p.1.contains(&key));

        if let Some(data) = p32 {
            let value = parse::<u32>(value, "integer")?;
            self.counts.push(Parameter {
                name: data.0,
                description: data.2,
                value,
            });
            return Ok(());
        }

        let data = TEXTS
            .iter()
            .find(|t| t.0 == key || t.1.contains(&key))
            .ok_or_else(|| ParParseError::UnrecognisedKey(key.to_string()))?;
        
        self.texts.push(Parameter {
            name: data.0,
            description: data.2,
            value: value.to_string(),
        });

        Ok(())
    }

    fn parse_special(
        &mut self, 
        parts: &[&str],
    ) -> Result<bool, ParParseError> {
        let key = parts[0];
        let value = parts[1];

        println!("Spec: '{}' '{}'", key, value);

        // Coords
        if ["RAJ", "RA"].contains(&key) {
            if self.ra.value != FittedCoord::Missing {
                return Err(ParParseError::RepeatPosition);
            }

            self.ra = Parameter {
                name: "RA",
                description: "J2000 right ascension",
                value: parse_coord(value, parts, true)?,
            };
            
            return Ok(true);
        }
        if ["DECJ", "DEC"].contains(&key) {
            if self.dec.value != FittedCoord::Missing { 
                return Err(ParParseError::RepeatPosition);
            }
            
            self.dec = Parameter {
                name: "DEC",
                description: "J2000 declination",
                value: parse_coord(value, parts, false)?,
            };
            
            return Ok(true);
        }

        // Which time ephemeris to use (IF99/FB90)
        if "TIMEEPH" == key {
            self.time_eph = match value {
                "IF99" => TimeEphemeris::IF99,
                "FB90" => TimeEphemeris::FB90,
                other => return Err(ParParseError::UnknownTimeEphemeris(other.to_string()))
            };
            return Ok(true);
        }

        // Binary model
        if "MODEL" == key {
            self.binary_model = match value {
                "BT" => BinaryModel::BT,
                "DD" => BinaryModel::DD,
                "ELL1" => BinaryModel::ELL1,
                "MSS" => BinaryModel::MSS,
                other => return Err(ParParseError::UnknownBinaryModel(other.to_string()))
            };
            return Ok(true);
        }

        // Method for transforming from terrestrial to celestial frame 
        if "T2CMETHOD" == key {
            self.t2c_method = match value {
                "TEMPO" => T2CMethod::TEMPO,
                "IAU2000B" => T2CMethod::IAU2000B,
                other => return Err(ParParseError::UnknownT2CMethod(other.to_string()))
            };
            return Ok(true);
        }

        // Units
        if "UNITS" == key {
            self.units = match value {
                "SI" => Units::SI,
                "TCD" => Units::TCD,
                "TDB" => Units::TDB,
                other => return Err(ParParseError::UnknownUnits(other.to_string()))
            };
            return Ok(true);
        }

        if "MODE" == key {
            self.error_mode = match value {
                "0" => ErrorMode::Mode0,
                "1" => ErrorMode::Mode1,
                other => return Err(ParParseError::UnknownErrorMode(other.to_string()))
            };
            return Ok(true);
        }

        Ok(false)
    }
    
    /// Performs a little check to see everything's ok.
    fn check(&mut self) -> Result<(), ParParseError> {
        // Check mandatory params
        if self.ra.value == FittedCoord::Missing 
        || self.dec.value == FittedCoord::Missing {
            return Err(ParParseError::NoPosition);
        }
        if self.texts
            .iter()
            .find(|t| t.name == "PSR")
            .is_none() {
            return Err(ParParseError::NoName);
        }
        if self.parameters
            .iter()
            .find(|t| t.name == "PEPOCH")
            .is_none() {
            return Err(ParParseError::NoPEpoch);
        }
        if self.parameters
            .iter()
            .find(|t| t.name == "F0")
            .is_none() {
            return Err(ParParseError::NoFrequency);
        }
        if self.parameters
            .iter()
            .find(|t| t.name == "DM")
            .is_none() {
            return Err(ParParseError::NoDispersion);
        }

        // Check for duplicates
        let p64dupes = find_duplicates(&self.parameters);
        if !p64dupes.is_empty() {
            return Err(ParParseError::DuplicateParameters(
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
            return Err(ParParseError::DuplicateParameters(
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
            return Err(ParParseError::DuplicateParameters(
                fdupes
                    .into_iter()
                    .map(|(i, j)| (
                        self.parameters[i].name.to_string(),
                        self.parameters[j].name.to_string(),
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

fn parse_coord(value: &str, parts: &[&str], is_ra: bool) -> Result<FittedCoord, ParParseError> {
    let coord_parts = value.split(":").collect::<Vec<_>>();
    if coord_parts.len() != 3 {
        return Err( match is_ra {
            true => ParParseError::InvalidRA(value.to_string()),
            false => ParParseError::InvalidDec(value.to_string()),
        });
    }
    
    let hrsdeg = parse::<i8>(
        coord_parts[0], 
        if is_ra {"hours [0, 24]"} else {"degrees [-90, 90]"}
    )?;
    let minutes = parse::<u8>(coord_parts[1], "minutes")?;
    let seconds = parse::<f64>(coord_parts[2], "seconds")?;

    match is_ra {
        true => 
            if hrsdeg < 0 
            || hrsdeg >= 24 
            || minutes >= 60 
            || seconds >= 60.0 {
                return Err(ParParseError::InvalidRA(value.to_string()));
            },
        false => 
            if hrsdeg < -90
            || hrsdeg == -90 && (minutes > 0 || seconds > 0.0) 
            || hrsdeg > 90 
            || hrsdeg == 90 && (minutes > 0 || seconds > 0.0)
            || minutes >= 60 
            || seconds >= 60.0 {
                return Err(ParParseError::InvalidRA(value.to_string()));
            },
    }

    let fit_info = if parts.len() > 3 {
        let fit = parse::<bool>(&parts[2], "bool")?;
        let error = parse::<f64>(&parts[3], "double")?;
        FittedCoord::FitInfo { value: (hrsdeg, minutes, seconds), fit, error }
    } else {
        FittedCoord::JustValue(hrsdeg, minutes, seconds)
    };

    Ok(fit_info)
}

fn find_duplicates<T>(params: &[Parameter<T>]) -> Vec<(usize, usize)> {
    params
        .iter()
        .enumerate()
        .filter_map(|(i, p1)| params[i+1..]
            .iter()
            .enumerate()
            .find(|(_, p2)| p1.name==p2.name)
            .map(|(j, _)| (i, j))
        ).collect()
}

fn parse_flag(flag: &str) -> Result<Parameter<()>, ParParseError> {
    FLAGS
        .iter()
        .find(|f| f.0 == flag || f.1.contains(&flag))
        .map(|data| Parameter {
            name: data.0,
            description: data.2,
            value: (),
        })
        .ok_or_else(|| ParParseError::UnknownFlag(flag.to_string()))
}

fn parse_param64(
    parts: &[&str],
) -> Result<Option<Parameter<FittedParameter>>, ParParseError> {
    let name = parts[0];
    
    let param = PARAMETERS
        .iter()
        .find(|p| p.0 == name || p.1.contains(&name))
        .map(|data| Parameter {
            name: data.0,
            description: data.2,
            value: FittedParameter::JustValue(0.0),
        });

    let mut param = match param {
        Some(p) => p,
        None => return Ok(None),
    };
    
    let value = parse::<f64>(&parts[1], "double")?;

    let fit_info = if parts.len() > 3 {
        let fit = parse::<bool>(&parts[2], "bool")?;
        let error = parse::<f64>(&parts[3], "double")?;
        FittedParameter::FitInfo { value, fit, error }
    } else {
        FittedParameter::JustValue(value)
    };
    param.value = fit_info;

    Ok(Some(param))
}

fn parse<T: std::str::FromStr>(
    value: &str, 
    type_name: &'static str
) -> Result<T, ParParseError> {
    value.parse()
        .map_err(|_| ParParseError::Unparsable { 
            value: value.to_string(), to_type: type_name 
        })
}

/// All documented parfile parameters with f64 values.
const PARAMETERS: &[(&str, &[&str], &str)] = &[
    ("F0", &[],                 "The rotational frequency (Hz)"),
    ("F1", &[],                 "The 1st time derivative of the rotational frequency (Hz / s)"),
    ("F2", &[],                 "The 2nd time derivative of the rotational frequency (Hz / s^2)"),
    ("F3", &[],                 "The 3rd time derivative of the rotational frequency (Hz / s^3)"),
    ("F4", &[],                 "The 4th time derivative of the rotational frequency (Hz / s^4)"),
    ("F5", &[],                 "The 5th time derivative of the rotational frequency (Hz / s^5)"),
    ("F6", &[],                 "The 6th time derivative of the rotational frequency (Hz / s^6)"),
    ("P0", &["P"],              "Spin period of pulsar (s)"),
    ("P1", &["PDOT"],           "Spin down rate of pulsar (10^-15)"),
    ("PEPOCH", &[],             "Epoch of period measurement (MJD)"),
    ("ELONG", &["LAMBDA"],      "Ecliptic longitude (deg)"),
    ("ELAT", &["BETA"],         "Ecliptic latitude (deg)"),
    ("POSEPOCH", &[],           "Epoch of position measurement (MJD)"),
    ("PMLAMBDA", &["PMELONG"],  "Proper motion in ecliptic longitude (mas/yr)"),
    ("PMBETA", &["PMELAT"],     "Proper motion in ecliptic latitude (mas/yr)"),
    ("PMRA", &[],               "Proper motion in right ascension (mas/yr)"),
    ("PMDEC", &[],              "proper motion in declination (mas/yr)"),
    ("DMEPOCH", &[],            "Epoch of DM measurement (MJD)"),
    ("DM", &[],                 "The dispersion measure (cm^-3 pc)"),
    ("DM1", &[],                "1st time derivative of the dispersion measure (cm^-3 pc / s)"),
    ("DM2", &[],                "2nd time derivative of the dispersion measure (cm^-3 pc / s^2)"),
    ("DM3", &[],                "3rd time derivative of the dispersion measure (cm^-3 pc / s^3)"),
    ("DM4", &[],                "4th time derivative of the dispersion measure (cm^-3 pc / s^4)"),
    ("DM5", &[],                "5th time derivative of the dispersion measure (cm^-3 pc / s^5)"),
    ("DM6", &[],                "6th time derivative of the dispersion measure (cm^-3 pc / s^6)"),
    ("FDD", &[],                "Frequency-dependent delay"),
    ("PX", &[],                 "Parallax (mas)"),
    ("PMRV", &[],               "Radial velocity"),
    ("WAVE_OM", &[],            "Frequency of fundamental sinusoid for whitening"),
    ("WAVE1", &[],              "Amplitude of sine and cosine for the 1st harmonic for whitening"),
    ("WAVE2", &[],              "Amplitude of sine and cosine for the 2nd harmonic for whitening"),
    ("WAVE3", &[],              "Amplitude of sine and cosine for the 3rd harmonic for whitening"),
    ("WAVE4", &[],              "Amplitude of sine and cosine for the 4th harmonic for whitening"),
    ("WAVE5", &[],              "Amplitude of sine and cosine for the 5th harmonic for whitening"),
    ("WAVE6", &[],              "Amplitude of sine and cosine for the 6th harmonic for whitening"),
    ("TRES", &[],               "Rms timing residual (µs)"),
    ("NE1AU", &[],              "The electron density at 1 AU due to the solar wind"),
    ("TZRMJD", &[],             "Missing info"),
    ("TZRFRQ", &[],             "Missing info"),
    ("START", &[],              "Missing info"),
    ("FINISH", &[],             "Missing info"),
    ("A1", &[],                 "Projected semi-major axis of orbit (lt-sec)"),
    ("PB", &[],                 "Orbital period (days)"),
    ("P1", &["PBDOT"],          "1st time derivative of binary period (days / s)"),
    ("PB2", &[],                "2nd time derivative of binary period (days / s^2)"),
    ("PB2", &[],                "3rd time derivative of binary period (days / s^3)"),
    ("PB3", &[],                "4th time derivative of binary period (days / s^4)"),
    ("PB5", &[],                "5th time derivative of binary period (days / s^5)"),
    ("PB6", &[],                "6th time derivative of binary period (days / s^6)"),
    ("ECC", &["E"],             "Eccentricity of orbit"),
    ("T0", &[],                 "Epoch of periastron (MJD)"),
    ("OM", &[],                 "Longitude of periastron (degrees)"),
    ("TASC", &[],               "Epoch of ascending node (MJD)"),
    ("EPS1", &[],               "ECC×sin(OM) for ELL1 model"),
    ("EPS2", &[],               "ECC×cos(OM) for ELL1 model"),
    ("OMDOT", &[],              "Rate of advance of periastron (deg/yr)"),
    ("A1DOT", &["XDOT"],        "Rate of change of projected semi-major axis (10^-12)"),
    ("SINI", &[],               "Sine of inclination angle"),
    ("M2", &[],                 "Companion mass (solar masses)"),
    ("XPBDOT", &[],             "Rate of change of orbital period minus GR prediction"),
    ("ECCDOT", &["EDOT"],       "Rate of change of eccentricity"),
    ("GAMMA", &[],              "Post-Keplerian ’gamma’ term (s)"),
    ("DR", &[],                 "Relativistic deformation of the orbit"),
    ("DTH", &[],                "Relativistic deformation of the orbit"),
    ("A0", &[],                 "Aberration parameter A0"),
    ("B0", &[],                 "Aberration parameter B0"),
    ("BP", &[],                 "Tensor multi-scalar parameter beta-prime"),
    ("BPP", &[],                "Tensor multi-scalar parameter beta-prime-prime"),
    ("DTHETA", &[],             "Relativistic deformation of the orbit"),
    ("XOMDOT", &[],             "Rate of periastron advance minus GR prediction (deg/yr)"),
    ("EPS1DOT", &[],            "Missing info"),
    ("EPS2DOT", &[],            "Missing info"),
    ("KOM", &[],                "Missing info"),
    ("KIN", &[],                "Missing info"),
    ("SHAPMAX", &[],            "Missing info"),
    ("MTOT", &[],               "Total system mass solar masses"),
    // No info so far on what these are...
    // ("BPJEP", &[],     "Missing info"),
    // ("BPJPH", &[],     "Missing info"),
    // ("BPJA1", &[],     "Missing info"),
    // ("BPJEC", &[],     "Missing info"),
    // ("BPJOM", &[],     "Missing info"),
    // ("BPJPB", &[],     "Missing info"),
];

/// All documented parfile parameters with u32 values.
const PARAMETERS_U32: &[(&str, &[&str], &str)] = &[
    ("NITS", &[],  "Number of iterations for the fitting routines"),
    ("IBOOT", &[], "Number of iterations used in the bootstrap fitting method"),
];

/// All documented parfile parameters with String values.
const TEXTS: &[(&str, &[&str], &str)] = &[
    ("PSR", &["PSRJ", "PSRB"],  "Definition of clock to use"),
    ("CLK", &[],                "Definition of clock to use"),
    ("CLK_CORR_CHAIN", &[],     "Clock correction chain(s) to use"),
    ("EPHEM", &[],              "Which solar system ephemeris to use"),
    ("TZRSITE", &[],            "Missing info"),
    ("NSPAN", &["TSPAN"],       "Missing info"),
    ("EPHVER", &[],             "Missing info"),
    ("TRACK", &[],              "Missing info"),
    ("AFAC", &[],               "Missing info"),
    ("DM_SERIES", &[],          "Missing info"),
];

/// All documented parfile flags.
const FLAGS: &[(&str, &[&str], &str)] = &[
    ("TEMPO1", &[],                 "Whether to run in tempo emulation mode: e.g. TDB units (Default=false)"),
    ("NOTRACK", &[],                "Switch oﬀ tracking mode"),
    ("NO_SS_SHAPIRO", &[],          "Switch oﬀ the calculation of the Solar system Shapiro delay"),
    ("IPM", &[],                    "Switch oﬀ calculation of the interplanetary medium"),
    ("DILATE_FREQ", &[],            "Whether or not to apply gravitational redshift and time dilation to observing frequency"),
    ("PLANET_SHAPIRO", &[],         "Missing info"),
    ("CORRECT_TROPOSPHERE", &[],    "Whether or not to apply tropospheric delay corrections"),
];
