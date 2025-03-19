use std::{fs::File, io::{BufRead, BufReader}, path::PathBuf};
use crate::error::{PsruError, TimContext};

pub use toa::*;

mod toa;
mod tests;

/// Reads a .tim file recursively. Returns errors for missing TOA values,
/// flags without values, and malformed entries.
/// 
/// Currently, the only implemented format is for Tempo2.
pub fn read_tim(path: PathBuf, format: TimFormat) -> Result<Vec<TOAInfo>, PsruError> {
    let mut toa_infos = Vec::new();

    let file = File::open(path.clone()).map_err(PsruError::IOError)?;
    let reader = BufReader::new(file);

    let directory = path.parent().unwrap().to_path_buf();
    let mut ctx = TimContext::new(&path.to_string_lossy(), 0);

    for (line_number, result) in reader.lines().enumerate() {
        let line = result.map_err(PsruError::IOError)?;
        if line.is_empty() { continue; }
        
        ctx.line(line_number + 1);

        parse_line(format, directory.clone(), &mut toa_infos, &line)
            .map_err(|err| err.set_tim_ctx(&ctx))?;
    }
    
    Ok(toa_infos)
}
    
fn parse_line(
    mode: TimFormat,
    mut directory: PathBuf,
    toa_infos: &mut Vec<TOAInfo>, 
    line: &str
) -> Result<(), PsruError> {
    let parts = line.split_whitespace().collect::<Vec<_>>();

    if parts[0] == "INCLUDE" {
        directory.push(parts[1]);
        let mut nested_tim = read_tim(directory, mode)?;
        
        toa_infos.append(&mut nested_tim);
        return Ok(())
    }

    if parts[0] == "FORMAT" && parts[1] == "1" {
        if mode != TimFormat::Tempo2 {
            return Err(PsruError::TimFormatDiscrepancy(
                None, String::from("Tempo2"))
            );
        }
        return Ok(())
    }

    if parts[0] == "MODE" && parts[1] == "1" {
        // I don't know what this means
        return Ok(())
    }

    let toa_info = match mode {
        TimFormat::Tempo2 => TOAInfo::parse_tempo2(&parts)?,
        TimFormat::Parkes => TOAInfo::parse_parkes(line)?,
    };

    toa_infos.push(toa_info);

    Ok(())
}

#[derive(Debug, PartialEq, Clone, Copy)]
/// The format used for parsing TOAs in .tim files.
pub enum TimFormat {
    Tempo2,
    /// Not implemented.
    Parkes,
}
