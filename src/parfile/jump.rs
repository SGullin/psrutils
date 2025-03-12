use super::{parse_bool, parse_f64, ParParseError};

/// Add a constant oﬀset between specified TOAs.
#[derive(Debug)]
pub struct Jump {
    selector: JumpType,
    value: f64,
    fit: bool,
}

#[derive(Debug)]
pub enum JumpType {
    MJD(f64, f64),
    FREQ(f64, f64),
    TEL(String),
    NAME(String),
    FLAG(String, String),
}

impl Jump {    
    /// This will parse a jump, which are written on one line. If anything is 
    /// missing or malformed, an error is returned.
    pub(crate) fn parse(parts: &[&str], jumps: &mut Vec<Jump>) -> Result<bool, ParParseError> {
        if parts[0] != "JUMP" {
            return Ok(false);
        }

        let es = parts.join(" ");
        let error = || ParParseError::IncompleteJump(es.clone());
        if parts.len() < 3 {
            return Err(error());
        }

        let mut parts = parts.iter();
        let selector = match *parts.next().unwrap() {
            "MJD" => JumpType::MJD(
                parse_f64(parts.next().ok_or_else(error)?)?,
                parse_f64(parts.next().ok_or_else(error)?)?,
            ),
            "FREQ" => JumpType::FREQ(
                parse_f64(parts.next().ok_or_else(error)?)?,
                parse_f64(parts.next().ok_or_else(error)?)?,
            ),
            "TEL" => JumpType::TEL(parts.next().ok_or_else(error)?.to_string()),
            "NAME" => JumpType::NAME(parts.next().ok_or_else(error)?.to_string()),
            
            flag => JumpType::FLAG(
                flag.to_string(), 
                parts.next().ok_or_else(error)?.to_string(),
            ),
        };

        let jump = Jump {
            selector,
            value: parse_f64(parts.next().ok_or_else(error)?)?,
            fit: parse_bool(parts.next().ok_or_else(error)?)?,
        };
        jumps.push(jump);

        Ok(true)
    }
    
    pub(crate) fn write(&self) -> String {
        let mut line = String::from("JUMP");
        match &self.selector {
            JumpType::MJD(v1, v2) => line += &format!("MJD {} {}", v1, v2),
            JumpType::FREQ(v1, v2) => line += &format!("FREQ {} {}", v1, v2),
            JumpType::TEL(id) => line += &format!("TEL {}", id),
            JumpType::NAME(name) => line += &format!("NAME {}", name),
            JumpType::FLAG(f, v) => line += &format!("{} {}", f, v),
        }

        line += &format!(" {} {}", self.value, if self.fit {"1"} else {"0"});

        line
    }
}