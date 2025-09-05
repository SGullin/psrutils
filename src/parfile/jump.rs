use super::PsruError;
use crate::parse_tools::parse_bool;
use crate::parse_tools::parse_f64;

/// Add a constant oﬀset between specified TOAs.
#[derive(Debug)]
pub struct Jump {
    /// What flag was used to id the jump arguments.
    pub jtype: JumpType,

    /// The jump size.
    pub value: f64,

    /// Fitting status.
    pub fit: bool,
}

#[derive(Debug)]
pub enum JumpType {
    Mjd(f64, f64),
    Freq(f64, f64),
    Tel(String),
    Name(String),
    Flag(String, String),
}

impl Jump {
    /// This will parse a jump, which are written on one line. If anything is
    /// missing or malformed, an error is returned.
    pub(crate) fn parse(
        parts: &[&str],
        jumps: &mut Vec<Self>,
    ) -> Result<bool, PsruError> {
        if parts[0] != "JUMP" {
            return Ok(false);
        }

        let es = parts.join(" ");
        let error = || PsruError::IncompleteJump(es.clone());
        if parts.len() < 3 {
            return Err(error());
        }

        let mut parts = parts.iter();
        _ = parts.next();
        let selector = match *parts.next().unwrap() {
            "MJD" => JumpType::Mjd(
                parse_f64(parts.next().ok_or_else(error)?)?,
                parse_f64(parts.next().ok_or_else(error)?)?,
            ),
            "FREQ" => JumpType::Freq(
                parse_f64(parts.next().ok_or_else(error)?)?,
                parse_f64(parts.next().ok_or_else(error)?)?,
            ),
            "TEL" => {
                JumpType::Tel((*parts.next().ok_or_else(error)?).to_string())
            }
            "NAME" => {
                JumpType::Name((*parts.next().ok_or_else(error)?).to_string())
            }

            flag => JumpType::Flag(
                flag.to_string(),
                (*parts.next().ok_or_else(error)?).to_string(),
            ),
        };

        let jump = Self {
            jtype: selector,
            value: parse_f64(parts.next().ok_or_else(error)?)?,
            fit: parse_bool(parts.next().ok_or_else(error)?)?,
        };
        jumps.push(jump);

        Ok(true)
    }

    pub(crate) fn write(&self) -> String {
        let mut line = String::from("JUMP");
        match &self.jtype {
            JumpType::Mjd(v1, v2) => line += &format!("MJD {v1} {v2}"),
            JumpType::Freq(v1, v2) => line += &format!("FREQ {v1} {v2}"),
            JumpType::Tel(id) => line += &format!("TEL {id}"),
            JumpType::Name(name) => line += &format!("NAME {name}"),
            JumpType::Flag(f, v) => line += &format!("{f} {v}"),
        }

        line +=
            &format!(" {} {}", self.value, if self.fit { "1" } else { "0" });

        line
    }
}
