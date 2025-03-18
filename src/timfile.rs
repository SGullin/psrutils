use std::{fs::File, io::{BufRead, BufReader}};
use toa::TOAInfo;

use crate::error::PsruError;

mod toa;
mod tests;

/// Reads a .tim file recursively. Returns errors for missing TOA values,
/// flags without values, and malformed entries.
pub fn read_tim(reader: impl BufRead) -> Result<Vec<TOAInfo>, PsruError> {
    let mut toa_infos = Vec::new();
    
    let mut mode = ReadingMode::Tempo2;

    for result in reader.lines() {
        let line = result.map_err(PsruError::IOError)?;
        if line.is_empty() { continue; }

        parse_line(&mut mode, &mut toa_infos, &line)?;
    }
    
    Ok(toa_infos)
}
    
fn parse_line(
    mode: &mut ReadingMode,
    toa_infos: &mut Vec<TOAInfo>, 
    line: &str
) -> Result<(), PsruError> {
    let parts = line.split_whitespace().collect::<Vec<_>>();

    if parts[0] == "INCLUDE" {
        let file = File::open(parts[1]).map_err(PsruError::IOError)?;
        let reader = BufReader::new(file);
        let mut nested_tim = read_tim(reader)?;
        
        toa_infos.append(&mut nested_tim);

        return Ok(())
    }

    if parts[0] == "FORMAT" && parts[1] == "1" {
        *mode = ReadingMode::Tempo2;
    }

    let toa_info = match mode {
        ReadingMode::Tempo2 => TOAInfo::parse_tempo2(&parts)?,
        ReadingMode::Parkes => TOAInfo::parse_parkes(&parts)?,
    };

    toa_infos.push(toa_info);

    Ok(())
}

enum ReadingMode {
    Tempo2,
    Parkes,
}
