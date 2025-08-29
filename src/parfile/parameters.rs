use std::str::FromStr;

use crate::data_types::J2000Coord;
use crate::error::PsruError;
use crate::parse_tools::{parse_f64, parse_bool, parse_u32};

type Result<T> = std::result::Result<T, PsruError>;

/// A parameter and information about whether it's being fit or not. 
#[derive(Debug, Default, PartialEq)]
pub enum FittedParameterValue<T> {
    /// For internal use.
    #[default]
    Missing,

    /// There's no information about it being fit or not.
    JustValue(T),
    /// There's fit information.
    FitInfo {
        /// The arrived at value.
        value: T,
        /// Whether to fit or not to fit.
        fit: bool,
        /// The uncertainty of the fit.
        error: f64,
    }    
}

#[derive(Debug, Default)]
/// An entry in a `.par` file. 
pub struct Parameter<T> {
    name: &'static str,
    description: &'static str,
    value: T,
}
impl<T> Parameter<T> {
    /// The name of the parameter, i.e. a functioning key, and not necessarily
    /// the most readable thing.
    pub const fn name(&self) -> &str { self.name }

    /// A description of the parameter, if it has one. Most entries are based 
    /// on the Tempo2 manual.
    pub const fn description(&self) -> &str { self.description }

    /// The value recorded.
    pub const fn value(&self) -> &T { &self.value }
    
    pub(crate) const fn new(
        data: &(&'static str, &[&str], &'static str), 
        value: T,
    ) -> Self {
        Self { 
            name: data.0,
            description: data.2, 
            value
        }
    }
}
impl<T> std::fmt::Display for Parameter<FittedParameterValue<T>> 
where T: std::fmt::Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.value {
            FittedParameterValue::Missing => write!(f, "MISSING"),
            FittedParameterValue::FitInfo { value, fit, error } => write!(
                f,
                "{} {} {} {}",
                self.name,
                value,
                if *fit {"1"} else {"0"},
                error,
            ),
            FittedParameterValue::JustValue(v) => write!(f, "{} {}", self.name, v),
        }
    }
}


pub type FittedParameter = Parameter<FittedParameterValue<f64>>;
pub type J2000Fit<T> = FittedParameterValue<J2000Coord<T>>;

pub(super) fn parse_coord<T>(
    value: &str, 
    parts: &[&str], 
) -> Result<J2000Fit<T>> 
where J2000Coord<T>: FromStr, 
    <J2000Coord<T> as FromStr>::Err: Into<PsruError>  {
    let coord = value.parse::<J2000Coord<T>>().map_err(Into::into)?;

    let fit_info = if parts.len() > 3 {
        let fit = parse_bool(parts[2])?;
        let error = parse_f64(parts[3])?;
        FittedParameterValue::FitInfo { value: coord, fit, error }
    } else {
        FittedParameterValue::JustValue(coord)
    };

    Ok(fit_info)
}

pub(super) fn parse_fitted(
    parts: &[&str],
) -> Result<Option<FittedParameter>> {
    let name = parts[0];
    
    let param = PARAMETERS
    .iter()
    .find(|p| p.0 == name || p.1.contains(&name))
    .map(|data| Parameter {
        name: data.0,
        description: data.2,
        value: FittedParameterValue::JustValue(0.0),
    });
    
    let Some(mut param) = param else { return Ok(None) };
    
    let value = parse_f64(parts[1])?;
    
    let fit_info = if parts.len() > 3 {
        let fit = parse_bool(parts[2])?;
        let error = parse_f64(parts[3])?;
        FittedParameterValue::FitInfo { value, fit, error }
    } else {
        FittedParameterValue::JustValue(value)
    };
    param.value = fit_info;
    
    Ok(Some(param))
}

pub(super) fn parse_flag(parts: &[&str]) -> Result<Option<Parameter<bool>>> {
    let name = parts[0];
    
    let flag = FLAGS
        .iter()
        .find(|p| p.0 == name || p.1.contains(&name))
        .map(|data| Parameter {
            name: data.0,
            description: data.2,
            value: true,
        });
   
    let Some(mut flag) = flag else { return Ok(None) };

    flag.value = parse_bool(parts[1])?;

    Ok(Some(flag))
}

pub(super) fn parse_count(parts: &[&str]) -> Result<Option<Parameter<u32>>> {
    let key = parts[0];
    let value = parts[1];
    let p32 = PARAMETERS_U32
        .iter()
        .find(|p| p.0 == key || p.1.contains(&key));
    
    if let Some(data) = p32 {
        let value = parse_u32(value)?;
        return Ok(Some(Parameter::new(data, value)));
    }
    Ok(None)
}

pub(super) fn parse_text(parts: &[&str]) -> Result<Option<Parameter<String>>> {
    let key = parts[0];
    let value = parts[1];

    TEXTS
        .iter()
        .find(|t| t.0 == key || t.1.contains(&key))
        .map(|data| Some(Parameter::new(data, value.to_string())))
        .ok_or_else(|| PsruError::ParUnrecognisedKey(key.to_string()))
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
    ("NE_SW", &[],              "Encountered in the wild"),
    ("CHI2R", &[],              "Encountered in the wild"),
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
    ("NTOA", &[],  "Number of TOAs"),
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
    ("DILATEFREQ", &[],             "Encountered in the wild"),
];

/// Special ones. These are the only ones to have duplicate aliases...
pub const COORDS: &[(&str, &[&str], &str)] = &[
    ("RA", &["RA", "RAJ"],     "J2000 right ascension"),
    ("DEC", &["DEC", "DECJ"],  "J2000 declination"),
];