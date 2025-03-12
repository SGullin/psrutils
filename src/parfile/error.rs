use std::error::Error;

#[derive(Debug)]
pub enum ParParseError {
    Unparsable{ value: String, to_type: &'static str },
    IOError(std::io::Error),
    
    InvalidRA(String),
    InvalidDec(String),
    
    UnknownFlag(String),
    UnrecognisedKey(String),
    UnknownBinaryModel(String),
    UnknownTimeEphemeris(String),
    UnknownT2CMethod(String),
    UnknownUnits(String),
    UnknownErrorMode(String),

    IncompleteJump(String),
    BadGlitch(usize),

    NoName,
    NoFrequency,
    NoPEpoch,
    NoPosition,
    NoDispersion,

    RepeatName,
    RepeatFrequency,
    RepeatPEpoch,
    RepeatPosition,
    RepeatDispersion,

    BadFrequency,
    BadPEpoch,
    
    DuplicateParameters(Vec<(String, String)>),
}
impl std::fmt::Display for ParParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParParseError::Unparsable { value, to_type } 
                => write!(f, "Impossible to parse '{}' into type {}.", value, to_type),
            ParParseError::IOError(error) 
                => write!(f, "IO error >> {}", error),

            ParParseError::InvalidRA(ra) 
                => write!(f, "Invalid RA string '{}'.", ra),
            ParParseError::InvalidDec(dec) 
                => write!(f, "Invalid DEC string '{}'.", dec),

            ParParseError::UnknownFlag(flag) 
                => write!(f, "Unknown flag '{}'.", flag),
            ParParseError::UnrecognisedKey(k) 
                => write!(f, "Unrecognised key '{}'.", k),
            ParParseError::UnknownBinaryModel(m) 
                => write!(f, "Unknown binary model '{}'.", m),
            ParParseError::UnknownTimeEphemeris(te) 
                => write!(f, "Unknown time ephemeris '{}'.", te),
            ParParseError::UnknownT2CMethod(t2cm) 
                => write!(f, "Unknown t2CMethod '{}'.", t2cm),
            ParParseError::UnknownErrorMode(em) 
                => write!(f, "Unknown error mode '{}'.", em),
            ParParseError::UnknownUnits(u) 
                => write!(f, "Unknown units '{}'.", u),

            ParParseError::IncompleteJump(j) 
                => write!(f, "Incomplete jump '{}'.", j),
            ParParseError::BadGlitch(g) 
                => write!(f, "Glitch with index {} is incomplete.", g),

            ParParseError::NoName 
                => write!(f, "Missing PSR parameter."),
            ParParseError::NoFrequency 
                => write!(f, "Missing F0 parameter."),
            ParParseError::NoPEpoch 
                => write!(f, "Missing PEPOCH parameter."),
            ParParseError::NoPosition 
                => write!(f, "Missing RA or DEC parameter."),
            ParParseError::NoDispersion
                => write!(f, "Missing DM parameter."),

            ParParseError::RepeatName 
                => write!(f, "Repeated PSR parameter."),
            ParParseError::RepeatFrequency 
                => write!(f, "Repeated F0 parameter."),
            ParParseError::RepeatPEpoch
                => write!(f, "Repeated PEPOCH parameter."),
            ParParseError::RepeatPosition
                => write!(f, "Repeated RA or DEC parameter."),
            ParParseError::RepeatDispersion
                => write!(f, "Repeated DM parameter."),

            ParParseError::BadPEpoch
                => write!(f, "Bad PEPOCH parameter."),
            ParParseError::BadFrequency
                => write!(f, "Bad F0 parameter."),

            ParParseError::DuplicateParameters(items) 
                => write!(
                    f, "There are duplicate parameters defined:{}", 
                    items
                        .iter()
                        .fold(String::new(), |a, (l1, l2)| 
                            format!("{}\n * '{}' and '{}'", a, l1, l2))
                        ),
        }
    }
}

impl Error for ParParseError {}
