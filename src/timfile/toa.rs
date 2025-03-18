use crate::error::PsruError;
use crate::parse_tools::*;

/// The basic information contained in a calculated TOA.
pub struct TOAInfo {
    pub is_bad: bool,
    pub file: String,

    pub frequency: f64,

    pub mjd_int: u32,
    pub mjd_frac: f64,
    pub error: f64,

    pub site_id: String,
    pub comment: String,
 
    pub flags: Vec<Flag>,
}
impl TOAInfo {
    pub(crate) fn parse_tempo2(parts: &[&str]) -> Result<Self, PsruError> {
        let is_bad = parts[0] == "c" || parts[0] == "C";
        let (mut comments, mut values): (Vec<&str>, Vec<&str>) = parts
            .iter()
            .partition(|w| w.starts_with("#") && w.len() > 1);
        
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
        let frequency = parse_f64(&freq_text)?;

        let mjd_text = values
            .next()
            .ok_or(PsruError::TimUnexpectedEOL(None))?
            .split(".")
            .collect::<Vec<_>>();
        if mjd_text.len() != 2 {
            return Err(PsruError::TimMalformedMJD(None));
        }
        let mjd_int = parse_u32(mjd_text[0])?;
        let mjd_frac = parse_f64(&format!("0.{}", mjd_text[1]))?;

        let err_text = values
            .next()
            .ok_or(PsruError::TimUnexpectedEOL(None))?;
        let error = parse_f64(&err_text)?;

        let site_id = values
            .next()
            .ok_or(PsruError::TimUnexpectedEOL(None))?
            .to_string();

        println!(
            "{} {} {}.{} {} {}",
            file,
            frequency,
            mjd_int, mjd_frac,
            error,
            site_id,
        );

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
            .collect::<Vec<_>>();


        Ok(Self {
            is_bad,
            file,
            frequency,
            mjd_int,
            mjd_frac,
            error,
            site_id,
            comment,
            flags,
        })
    }
    
    pub(crate) fn parse_parkes(parts: &[&str]) -> Result<Self, PsruError> {
        todo!()
    }
}

fn parse_flag(key: &str, value: &str) -> Flag {
    let key = 
    if key.starts_with("-") {
        key[1..].to_string()
    } 
    else {
        key.to_string()
    };

    let value = match parse_f64(value) {
        Ok(v) => FlagValue::Double(v),
        Err(_) => FlagValue::String(value.to_string()),
    };

    Flag {
        key,
        value,
    }
}

pub struct Flag {
    pub key: String,
    pub value: FlagValue,
}
pub enum FlagValue {
    Double(f64),
    String(String),
}
