use super::PsruError;
use crate::parse_tools::parse_f64;

/// The data representing a glitch. The index, `number`, is kept as-is 
/// from the source, but it should be noted that the reader expects a 
/// non-disjuct range of indices, once everything's been read.
/// 
/// If e.g. "GLF1_2" shows up, we assume there should also be a glitch 1 
/// before it, but perhaps written later, so we presumptiously add it. If 
/// there is no glitch 1, that produces a warning in the end of the `read`
/// function. If there is not enough data to fully define a glitch, it is 
/// removed and a warning is issued.
#[derive(Debug, Default, Clone)]
pub struct Glitch {
    /// The index used in the file
    pub number: usize,
    /// Glitch epoch (MJD)
    pub epoch: f64,
    /// Glitch phase increment
    pub phase: f64,
    /// Glitch permanent pulse frequency increment (Hz)
    pub f0: f64,
    /// Glitch permanent frequency derivative increment (s^-2)
    pub f1: f64,
    /// Glitch pulse frequency increment (Hz)
    pub f0d: f64,
    /// Glitch Decay time constant (Hz)
    pub td: f64,
}

impl Glitch {
    /// This will parse one glitch parameter, since there does not seem to be 
    /// any restrictions on where these paramaters may occur in the file.
    pub(crate) fn parse(parts: &[&str], glitches: &mut Vec<Glitch>) -> Result<bool, PsruError> {
        let p0ps = parts[0].split("_").collect::<Vec<_>>();
        if p0ps.len() != 2 {
            return Ok(false);
        }

        if !["glep","glph","glf0","glf1","glf0d","gltd"].contains(&p0ps[0]) {
            return Ok(false);
        }
        
        let index = p0ps[1]
        .parse::<usize>()
        .map_err(|_| PsruError::Unparsable { 
            value: p0ps[0].to_string(), 
            to_type: "glitch index",
        })?;
        let value = parse_f64(parts[1])?;

        // Make sure there are glitches for all indicated slots...
        while glitches.len() <= index {
            glitches.push(Glitch::default());
        }

        let glitch = glitches.get_mut(index).unwrap();

        match p0ps[0] {
            "GLEP" => glitch.epoch = value,
            "GLPH" => glitch.phase = value,
            "GLF0" => glitch.f0 = value,
            "GLF1" => glitch.f1 = value,
            "GLF0D" => glitch.f0d = value,
            "GLTD" => glitch.td = value,

            _ => unreachable!(),
        }

        Ok(true)
    }
    
    /// Checks if the glitch is defined enough.
    pub(crate) fn check(&self) -> Result<(), PsruError> {
        if self.f0 == 0.0 
        || self.f0d == 0.0 
        || self.epoch == 0.0 {
            return Err(PsruError::BadGlitch(self.number));
        }

        Ok(())
    }
    
    pub(crate) fn write(&self) -> String {
        format!("
            GLEP_{0}  {1}\n\
            GLPH_{0}  {2}\n\
            GLF0_{0}  {3}\n\
            GLF1_{0}  {4}\n\
            GLF0D_{0} {5}\n\
            GLTD_{0}  {6}\n",
            self.number, 
            self.epoch,
            self.phase,
            self.f0,
            self.f1,
            self.f0d,
            self.td,
        )
    }
}