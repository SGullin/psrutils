use std::{fs::File, io::{BufRead, BufReader}, path::PathBuf};
use toa::TOAInfo;

use crate::error::{PsruError, TimContext};

mod toa;
mod tests;

/// Reads a .tim file recursively. Returns errors for missing TOA values,
/// flags without values, and malformed entries.
pub fn read_tim(path: &PathBuf) -> Result<Vec<TOAInfo>, PsruError> {
    let mut toa_infos = Vec::new();
    let file = File::open(path).map_err(PsruError::IOError)?;
    let reader = BufReader::new(file);

    let mut mode = ReadingMode::Tempo2;
    let directory = path.clone().parent().unwrap().to_path_buf();
    let mut ctx = TimContext::new(&path.to_string_lossy(), 0);

    for (line_number, result) in reader.lines().enumerate() {
        let line = result.map_err(PsruError::IOError)?;
        if line.is_empty() { continue; }
        
        ctx.line(line_number + 1);
        println!("ctx: {:?}", ctx);
        
        parse_line(&mut mode, &directory, &mut toa_infos, &line)
            .map_err(|err| err.set_tim_ctx(&ctx))?;
    }
    
    Ok(toa_infos)
}
    
fn parse_line(
    mode: &mut ReadingMode,
    directory: &PathBuf,
    toa_infos: &mut Vec<TOAInfo>, 
    line: &str
) -> Result<(), PsruError> {
    let parts = line.split_whitespace().collect::<Vec<_>>();
    println!("{:?}", parts);

    if parts[0] == "INCLUDE" {
        let mut path = directory.clone();
        path.push(parts[1]);
        let mut nested_tim = read_tim(&path)?;
        
        toa_infos.append(&mut nested_tim);
        return Ok(())
    }

    if parts[0] == "FORMAT" && parts[1] == "1" {
        *mode = ReadingMode::Tempo2;
        return Ok(())
    }

    if parts[0] == "MODE" && parts[1] == "1" {
        // I don't know what this means
        return Ok(())
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
