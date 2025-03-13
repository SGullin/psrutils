use super::error::ParParseError;
type Result<T> = std::result::Result<T, ParParseError>;

#[derive(Debug, Default, PartialEq)]
pub enum FittedParameterValue<T> {
    #[default]
    Missing,

    JustValue(T),
    FitInfo {
        value: T,
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
    
    pub(crate) fn new(
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
impl std::fmt::Display for Parameter<FittedParameterValue<f64>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.value {
            FittedParameterValue::Missing => write!(f, "MISSING"),
            FittedParameterValue::FitInfo { value, fit, error } => write!(
                f,
                "{} {} {} {}\n",
                self.name,
                value,
                if fit {"1"} else {"0"},
                error,
            ),
            FittedParameterValue::JustValue(v) => write!(f, "{} {}\n", self.name, v),
        }
    }
}
impl std::fmt::Display for Parameter<J2000Coord> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.value {
            FittedParameterValue::Missing => write!(f, "MISSING PARAMETER"),
            FittedParameterValue::FitInfo { value, fit, error } => write!(
                f,
                "{} {}:{}:{} {} {}\n",
                self.name,
                value.0, value.1, value.2,
                if fit {"1"} else {"0"},
                error,
            ),
            FittedParameterValue::JustValue((v1, v2, v3)) 
                => write!(f, "{} {}:{}:{}\n", self.name, v1, v2, v3),
        }
    }
}

pub type FittedParameter = Parameter<FittedParameterValue<f64>>;
pub type J2000Coord = FittedParameterValue<(i8, u8, f64)>;

pub(super) fn parse_dec(
    value: &str, 
    parts: &[&str], 
) -> Result<J2000Coord> {
    let coord_parts = value.split(":").collect::<Vec<_>>();
    if coord_parts.len() != 3 {
        return Err(ParParseError::InvalidDec(value.to_string()))
    }
    
    let degrees = coord_parts[0]
        .parse::<i8>()
        .map_err(|_| ParParseError::Unparsable { 
            value: coord_parts[0].to_string(), 
            to_type: "degrees [-90, 90]",
        })?;
    let minutes = coord_parts[1]
        .parse::<u8>()
        .map_err(|_| ParParseError::Unparsable { 
            value: coord_parts[0].to_string(), 
            to_type: "minutes",
        })?;
    let seconds = parse_f64(coord_parts[2])?;

    if degrees < -90
    || degrees == -90 && (minutes > 0 || seconds > 0.0) 
    || degrees > 90 
    || degrees == 90 && (minutes > 0 || seconds > 0.0)
    || minutes >= 60 
    || seconds >= 60.0 {
        return Err(ParParseError::InvalidRA(value.to_string()));
    }

    let fit_info = if parts.len() > 3 {
        let fit = parse_bool(&parts[2])?;
        let error = parse_f64(&parts[3])?;
        FittedParameterValue::FitInfo { value: (degrees, minutes, seconds), fit, error }
    } else {
        FittedParameterValue::JustValue((degrees, minutes, seconds))
    };

    Ok(fit_info)
}

pub(super) fn parse_ra(
    value: &str, 
    parts: &[&str], 
) -> Result<J2000Coord> {
    let coord_parts = value.split(":").collect::<Vec<_>>();
    if coord_parts.len() != 3 {
        return Err(ParParseError::InvalidRA(value.to_string()));
    }
    
    let hours = coord_parts[0]
        .parse::<i8>()
        .map_err(|_| ParParseError::Unparsable { 
            value: coord_parts[0].to_string(), 
            to_type: "hours [0, 24]"
        })?;
    let minutes = coord_parts[1]
        .parse::<u8>()
        .map_err(|_| ParParseError::Unparsable { 
            value: coord_parts[0].to_string(), 
            to_type: "minutes",
        })?;
    let seconds = parse_f64(coord_parts[2])?;

    if hours >= 24 
    || hours < 0 
    || minutes >= 60 
    || seconds >= 60.0 {
        return Err(ParParseError::InvalidRA(value.to_string()));
    }

    let fit_info = if parts.len() > 3 {
        let fit = parse_bool(&parts[2])?;
        let error = parse_f64(&parts[3])?;
        FittedParameterValue::FitInfo { value: (hours, minutes, seconds), fit, error }
    } else {
        FittedParameterValue::JustValue((hours, minutes, seconds))
    };

    Ok(fit_info)
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
   
    let mut flag = match flag {
        Some(p) => p,
        None => return Ok(None),
    };

    if parts.len() < 2 {
        return Err(ParParseError::FlagMissingValue(name.to_string()));
    }
    flag.value = parse_bool(parts[1])?;

    Ok(Some(flag))
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

    let mut param = match param {
        Some(p) => p,
        None => return Ok(None),
    };
    
    let value = parse_f64(&parts[1])?;

    let fit_info = if parts.len() > 3 {
        let fit = parse_bool(&parts[2])?;
        let error = parse_f64(&parts[3])?;
        FittedParameterValue::FitInfo { value, fit, error }
    } else {
        FittedParameterValue::JustValue(value)
    };
    param.value = fit_info;

    Ok(Some(param))
}

pub(super) fn parse_f64(value: &str) -> Result<f64> {
    value.parse()
        .map_err(|_| ParParseError::Unparsable { 
            value: value.to_string(), to_type: "double"
        })
}

pub(super) fn parse_u32(value: &str) -> Result<u32> {
    value.parse()
        .map_err(|_| ParParseError::Unparsable { 
            value: value.to_string(), to_type: "integer"
        })
}

pub(super) fn parse_bool(value: &str) -> Result<bool> {
    match value {
        "1" | "Y" => Ok(true),
        "0" | "N "=> Ok(false),
        _ => Err(ParParseError::Unparsable { 
            value: value.to_string(), to_type: "bool" 
        })
    }
}

/// All documented parfile parameters with f64 values.
pub(crate) const PARAMETERS: &[(&str, &[&str], &str)] = &[
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
    // No info so far on what these are...
    // ("BPJEP", &[],     "Missing info"),
    // ("BPJPH", &[],     "Missing info"),
    // ("BPJA1", &[],     "Missing info"),
    // ("BPJEC", &[],     "Missing info"),
    // ("BPJOM", &[],     "Missing info"),
    // ("BPJPB", &[],     "Missing info"),
];

/// All documented parfile parameters with u32 values.
pub(crate) const PARAMETERS_U32: &[(&str, &[&str], &str)] = &[
    ("NITS", &[],  "Number of iterations for the fitting routines"),
    ("IBOOT", &[], "Number of iterations used in the bootstrap fitting method"),
];

/// All documented parfile parameters with String values.
pub(crate) const TEXTS: &[(&str, &[&str], &str)] = &[
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
pub(crate) const FLAGS: &[(&str, &[&str], &str)] = &[
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
pub(crate) const COORDS: &[(&str, &[&str], &str)] = &[
    ("RA", &["RA", "RAJ"],     "J2000 right ascension"),
    ("DEC", &["DEC", "DECJ"],  "J2000 declination"),
];