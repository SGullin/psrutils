use crate::error::PsruError;
type Result<T> = std::result::Result<T, PsruError>;

pub fn parse_f64(value: &str) -> Result<f64> {
    value.parse().map_err(|_| PsruError::Unparsable {
        value: value.to_string(), to_type: "double"
    })
}

pub fn parse_u32(value: &str) -> Result<u32> {
    value.parse().map_err(|_| PsruError::Unparsable { 
        value: value.to_string(), to_type: "integer"
    })
}

pub fn parse_bool(value: &str) -> Result<bool> {
    match value {
        "1" | "Y" | "y" => Ok(true),
        "0" | "N" | "n" => Ok(false),
        _ => Err(PsruError::Unparsable { 
            value: value.to_string(), to_type: "bool" 
        })
    }
}
