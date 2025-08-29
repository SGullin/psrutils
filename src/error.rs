//! Errors enum.
#![allow(missing_docs)]

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
    OrphanFile,
    TimUnexpectedEOL(Option<TimContext>),
    TimMalformedMJD(Option<TimContext>),
    TimUnvaluedFlag(Option<TimContext>, String),
    TimFormatDiscrepancy(Option<TimContext>, String),
    TimNotAscii(Option<TimContext>),
    TimParkesMissingBlank(Option<TimContext>),
    TimParkesMissingPeriod(Option<TimContext>),
}
impl std::fmt::Display for PsruError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unparsable { value, to_type } 
                => write!(f, "Impossible to parse '{value}' into type {to_type}."),
            Self::IOError(error) 
                => write!(f, "IO error >> {error}"),

            Self::InvalidRA(ra) 
                => write!(f, "Invalid RA string '{ra}'."),
            Self::InvalidDec(dec) 
                => write!(f, "Invalid DEC string '{dec}'."),
            Self::ParMissingValue(p) 
                => write!(f, "Param '{p}' missing value."),

            Self::ParUnknownFlag(flag) 
                => write!(f, "Unknown flag '{flag}'."),
            Self::ParUnrecognisedKey(k) 
                => write!(f, "Unrecognised key '{k}'."),
            Self::UnknownBinaryModel(m) 
                => write!(f, "Unknown binary model '{m}'."),
            Self::UnknownTimeEphemeris(te) 
                => write!(f, "Unknown time ephemeris '{te}'."),
            Self::UnknownT2CMethod(t2cm) 
                => write!(f, "Unknown t2CMethod '{t2cm}'."),
            Self::UnknownErrorMode(em) 
                => write!(f, "Unknown error mode '{em}'."),
            Self::UnknownUnits(u) 
                => write!(f, "Unknown units '{u}'."),

            Self::IncompleteJump(j) 
                => write!(f, "Incomplete jump '{j}'."),
            Self::BadGlitch(g) 
                => write!(f, "Glitch with index {g} is incomplete."),

            Self::ParNoName 
                => write!(f, "Missing PSR parameter."),
            Self::ParNoFrequency 
                => write!(f, "Missing F0 parameter."),
            Self::ParNoPEpoch 
                => write!(f, "Missing PEPOCH parameter."),
            Self::ParNoDispersion
                => write!(f, "Missing DM parameter."),

            Self::ParBadPEpoch
                => write!(f, "Bad PEPOCH parameter."),
            Self::ParBadFrequency
                => write!(f, "Bad F0 parameter."),

            Self::ParDuplicateParameters(items) 
                => write!(
                    f, "There are duplicate parameters defined:{}", 
                    items
                        .iter()
                        .fold(String::new(), |a, (l1, l2)| 
                            format!("{a}\n * '{l1}' and '{l2}'"))
                        ),
            
            Self::ParRepeatParam(param)
                => write!(f, "Repeated '{param}' parameter."),

            Self::OrphanFile => write!(f,
                "File does not have a (readable) parent directory."),
            Self::TimUnexpectedEOL(ctx) => write!(f, 
                "{} TOA line ended prematurely",
                tim_ctx(ctx.as_ref())),
            Self::TimMalformedMJD(ctx) => write!(f,
                "{} MJD is expected to be in decimal format.",
                tim_ctx(ctx.as_ref())),
            Self::TimUnvaluedFlag(ctx, flag) => write!(f,
                "{} Flag '{}' did not have a value.", 
                tim_ctx(ctx.as_ref()), flag),
            Self::TimFormatDiscrepancy(ctx, fmt) => write!(f,
                "{} Read format does not match supplied '{}'", 
                tim_ctx(ctx.as_ref()), fmt),
            Self::TimNotAscii(ctx) => write!(f,
                "{} Cannot handle non-ascii text in the supplied mode.",
                tim_ctx(ctx.as_ref())),
            Self::TimParkesMissingBlank(ctx) => write!(f,
                "{} There's supposed to be a blank space in the first column.",
                tim_ctx(ctx.as_ref())),
            Self::TimParkesMissingPeriod(ctx) => write!(f,
                "{} There's supposed to be a period in the column 42.",
                tim_ctx(ctx.as_ref())),
        }
    }
}
impl PsruError {
    pub(crate) fn set_tim_ctx(mut self, ctx: &TimContext) -> Self {
        let (
            Self::TimUnexpectedEOL(old_ctx) |
            Self::TimNotAscii(old_ctx) |
            Self::TimMalformedMJD(old_ctx) |
            Self::TimUnvaluedFlag(old_ctx, _) |
            Self::TimFormatDiscrepancy(old_ctx, _)
        ) = &mut self else { return self };

        if old_ctx.is_none() {
            *old_ctx = Some(ctx.clone());
        }

        self
    }
}
impl Error for PsruError {}
impl From<std::io::Error> for PsruError {
    fn from(value: std::io::Error) -> Self {
        Self::IOError(value)
    }
}

#[derive(Debug, Clone)]
pub struct TimContext {
    fname: String,
    line: usize,
}
impl TimContext {
    pub(crate) fn new(fname: &str, line_number: usize) -> Self {
        Self { fname: fname.to_string(), line: line_number }
    }
    pub(crate) fn line(&mut self, number: usize) {
        self.line = number;
    }
}
impl std::fmt::Display for TimContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "In file '{}' on line {}:", self.fname, self.line)
    }
}

fn tim_ctx(ctx: Option<&TimContext>) -> String {
    ctx.map_or_else(String::new, TimContext::to_string)
}
