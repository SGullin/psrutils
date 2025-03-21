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

            PsruError::TimUnexpectedEOL(ctx) => write!(f, 
                "{} TOA line ended prematurely",
                tim_ctx(ctx)),
            PsruError::TimMalformedMJD(ctx) => write!(f,
                "{} MJD is expected to be in decimal format.",
                tim_ctx(ctx)),
            PsruError::TimUnvaluedFlag(ctx, flag) => write!(f,
                "{} Flag '{}' did not have a value.", 
                tim_ctx(ctx), flag),
            PsruError::TimFormatDiscrepancy(ctx, fmt) => write!(f,
                "{} Read format does not match supplied '{}'", 
                tim_ctx(ctx), fmt),
            PsruError::TimNotAscii(ctx) => write!(f,
                "{} Cannot handle non-ascii text in the supplied mode.",
                tim_ctx(ctx)),
            PsruError::TimParkesMissingBlank(ctx) => write!(f,
                "{} There's supposed to be a blank space in the first column.",
                tim_ctx(ctx)),
            PsruError::TimParkesMissingPeriod(ctx) => write!(f,
                "{} There's supposed to be a period in the column 42.",
                tim_ctx(ctx)),
        }
    }
}
impl PsruError {
    pub(crate) fn set_tim_ctx(mut self, ctx: &TimContext) -> PsruError {
        let old_ctx = match &mut self {
            PsruError::TimUnexpectedEOL(ctx) => ctx,
            PsruError::TimMalformedMJD(ctx) => ctx,
            PsruError::TimUnvaluedFlag(ctx, _) => ctx,
            PsruError::TimFormatDiscrepancy(ctx, _) => ctx,
            PsruError::TimNotAscii(ctx) => ctx,
            _ => return self,
        };

        if old_ctx.is_none() {
            *old_ctx = Some(ctx.clone());
        }

        self
    }
}
impl Error for PsruError {}

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

fn tim_ctx(ctx: &Option<TimContext>) -> String {
    match ctx {
        Some(ctx) => ctx.to_string(),
        None => String::new(),
    }
}
