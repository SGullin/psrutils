use std::collections::HashMap;

use crate::data_types::Mjd;
use crate::error::PsruError;
use crate::parse_tools::parse_f64;

#[derive(Debug, PartialEq)]
/// The basic information contained in a calculated TOA.
pub struct TOAInfo {
    /// Whether the TOA is marked as bad.
    pub is_bad: bool,
    /// The original file the TOA came from.
    pub file: String,

    /// Observation frequency.
    pub frequency: f64,

    /// The date-time.
    pub mjd: Mjd,
    /// The error in MJD.
    pub mjd_error: f64,

    /// The observation site identifier.
    pub site_id: String,
    /// Any comments left in the line.
    pub comment: String,
 
    /// All flags found. Which flags are used depends on the file's creator, 
    /// but they are all put as either of two versions: `f64` and `String`. 
    pub flags: HashMap<String, Flag>,
}
impl TOAInfo {
    /// Parses a single line of .tim-file information in TEMPO2-style.
    /// 
    /// # Errors 
    /// Returns errors for any malformed parameters.
    /// 
    /// ```
    /// # use psrutils::timfile::{TOAInfo, Flag};
    /// # use psrutils::data_types::Mjd;
    /// # use std::collections::HashMap;
    /// let line = "fname 1.0 55.0 0.0 st -flag value -flag2 42";
    /// let info = TOAInfo::from_line_tempo2(line).unwrap();
    /// let toa = TOAInfo {
    ///     is_bad: false,
    ///     file: String::from("fname"),
    ///     frequency: 1.0,
    ///     mjd: Mjd::new(55, 0.0),
    ///     mjd_error: 0.0,
    ///     site_id: String::from("st"),
    ///     flags: HashMap::from([
    ///         (String::from("flag"), Flag::String(String::from("value"))),
    ///         (String::from("flag2"), Flag::Double(42.0)),
    ///     ]),
    ///     comment: String::new(),
    /// };
    /// assert_eq!(info, toa);
    /// ```
    pub fn from_line_tempo2(line: &str) -> Result<Self, PsruError> {
        let parts = line.split_ascii_whitespace().collect::<Vec<_>>();
        Self::parse_tempo2(&parts)
    }

    /// Reads in tempo2 format. Comments are a little more allwoing than should
    /// be...
    pub(crate) fn parse_tempo2(parts: &[&str]) -> Result<Self, PsruError> {
        let is_bad = parts[0] == "c" || parts[0] == "C";
        let (mut comments, mut values): (Vec<&str>, Vec<&str>) = parts
            .iter()
            .partition(|w| w.starts_with('#') && w.len() > 1);
        
        if let Some(pos) = values.iter().position(|w| *w == "#") {
            values
                .split_off(pos)[1..]
                .iter()
                .for_each(|c| comments.push(c));
        }
        
        let comment = comments.join(" -- ");
        let mut values = values.into_iter();

        if is_bad { _ = values.next(); }

        let file = values
            .next()
            .ok_or(PsruError::TimUnexpectedEOL(None))?
            .to_string();
        
        let freq_text = values
            .next()
            .ok_or(PsruError::TimUnexpectedEOL(None))?;
        let frequency = parse_f64(freq_text)?;

        let mjd_text = values
            .next()
            .ok_or(PsruError::TimUnexpectedEOL(None))?;
        let mjd = mjd_text.parse::<Mjd>()?;

        let err_text = values
            .next()
            .ok_or(PsruError::TimUnexpectedEOL(None))?;
        let error = parse_f64(err_text)?;

        let site_id = values
            .next()
            .ok_or(PsruError::TimUnexpectedEOL(None))?
            .to_string();

        // Flags come in key-value pairs
        let remains = values.collect::<Vec<_>>();
        let chunks = remains.chunks_exact(2);
        if !chunks.remainder().is_empty() {
            return Err(PsruError::TimUnvaluedFlag(
                None,
                chunks.remainder()[0].to_string()
            ));
        }

        let flags = chunks
            .map(|s| parse_flag(s[0], s[1]))
            .collect::<HashMap<String, Flag>>();

        Ok(Self {
            is_bad,
            file,
            frequency,
            mjd,
            mjd_error: error,
            site_id,
            comment,
            flags,
        })
    }
    
    /// Not fully implemented.
    pub(crate) fn parse_parkes(line: &str) -> Result<Self, PsruError> {
        if !line.is_ascii() {
            return Err(PsruError::TimNotAscii(None));
        }

        if &line[0..1] != " " {
            return Err(PsruError::TimParkesMissingBlank(None));
        }
        if &line[41..42] != "." {
            return Err(PsruError::TimParkesMissingPeriod(None));
        }
        // let freq_text = &line[25..34];
        // let toa_int = &line[34..41];
        // let toa_frac = &line[42..54];
        // let phase = &line[55..62];
        // let toa_err = &line[63..70];
        // let observatory = &line[79..80];

        todo!()
    }
}

fn parse_flag(key: &str, value: &str) -> (String, Flag) {
    let key = key
        .strip_prefix('-')
        .map_or_else(|| key.to_string(), str::to_string);

    let value = parse_f64(value).map_or_else(
        |_| Flag::String(value.to_string()), 
        Flag::Double,
    );

    (
        key,
        value,
    )
}

#[derive(Debug, PartialEq)]
/// A TOA flag value.
pub enum Flag {
    /// Double precision value, or integers, if present.
    Double(f64),
    /// Anything that could not be cast to `f64`.
    String(String),
}
