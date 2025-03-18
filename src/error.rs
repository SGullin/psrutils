use std::error::Error;

#[derive(Debug)]
pub enum PsruError {
    Unparsable{ value: String, to_type: &'static str },
    IOError(std::io::Error),
    
    // Par errors ---------------------------------
    InvalidRA(String),
    InvalidDec(String),
    ParMissingValue(String),
    
    ParUnknownFlag(String),
    ParUnrecognisedKey(String),
    UnknownBinaryModel(String),
    UnknownTimeEphemeris(String),
    UnknownT2CMethod(String),
    UnknownUnits(String),
    UnknownErrorMode(String),

    IncompleteJump(String),
    BadGlitch(usize),

    ParNoName,
    ParNoFrequency,
    ParNoPEpoch,
    ParNoDispersion,

    ParBadFrequency,
    ParBadPEpoch,
    
    ParDuplicateParameters(Vec<(String, String)>),
    ParRepeatParam(String),

    // Tim errors ---------------------------------
    TimNotFormat1,
    TimUnexpectedEOL,
    TimMalformedMJD,
    TimUnvaluedFlag(String),
}
impl std::fmt::Display for PsruError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PsruError::Unparsable { value, to_type } 
                => write!(f, "Impossible to parse '{}' into type {}.", value, to_type),
            PsruError::IOError(error) 
                => write!(f, "IO error >> {}", error),

            PsruError::InvalidRA(ra) 
                => write!(f, "Invalid RA string '{}'.", ra),
            PsruError::InvalidDec(dec) 
                => write!(f, "Invalid DEC string '{}'.", dec),
            PsruError::ParMissingValue(p) 
                => write!(f, "Param '{}' missing value.", p),

            PsruError::ParUnknownFlag(flag) 
                => write!(f, "Unknown flag '{}'.", flag),
            PsruError::ParUnrecognisedKey(k) 
                => write!(f, "Unrecognised key '{}'.", k),
            PsruError::UnknownBinaryModel(m) 
                => write!(f, "Unknown binary model '{}'.", m),
            PsruError::UnknownTimeEphemeris(te) 
                => write!(f, "Unknown time ephemeris '{}'.", te),
            PsruError::UnknownT2CMethod(t2cm) 
                => write!(f, "Unknown t2CMethod '{}'.", t2cm),
            PsruError::UnknownErrorMode(em) 
                => write!(f, "Unknown error mode '{}'.", em),
            PsruError::UnknownUnits(u) 
                => write!(f, "Unknown units '{}'.", u),

            PsruError::IncompleteJump(j) 
                => write!(f, "Incomplete jump '{}'.", j),
            PsruError::BadGlitch(g) 
                => write!(f, "Glitch with index {} is incomplete.", g),

            PsruError::ParNoName 
                => write!(f, "Missing PSR parameter."),
            PsruError::ParNoFrequency 
                => write!(f, "Missing F0 parameter."),
            PsruError::ParNoPEpoch 
                => write!(f, "Missing PEPOCH parameter."),
            PsruError::ParNoDispersion
                => write!(f, "Missing DM parameter."),

            PsruError::ParBadPEpoch
                => write!(f, "Bad PEPOCH parameter."),
            PsruError::ParBadFrequency
                => write!(f, "Bad F0 parameter."),

            PsruError::ParDuplicateParameters(items) 
                => write!(
                    f, "There are duplicate parameters defined:{}", 
                    items
                        .iter()
                        .fold(String::new(), |a, (l1, l2)| 
                            format!("{}\n * '{}' and '{}'", a, l1, l2))
                        ),
            
            PsruError::ParRepeatParam(param)
                => write!(f, "Repeated '{}' parameter.", param),

            
            PsruError::TimNotFormat1 => write!(f, 
                "Currently only TEMPO2 format (FORMAT 1) is supported."),
            PsruError::TimUnexpectedEOL => write!(f, 
                "TOA line ended prematurely"),
            PsruError::TimMalformedMJD => write!(f,
                "MJD is expected to be in decimal format."),
            PsruError::TimUnvaluedFlag(flag) => write!(f,
                "Flag '{}' did not have a value.", flag),
        }
    }
}

impl Error for PsruError {}
