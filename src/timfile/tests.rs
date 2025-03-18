#[allow(unused)]
use std::io::{BufReader, LineWriter};
#[allow(unused)]
use super::*;

const MINIMAL: &[&str] = &[
    "dir/file.ext",
    "999.999",
    "55000.97531",
    "99.11",
    "tele-id",
];

#[test]
fn incomplete() {
    for i in 0..MINIMAL.len() { 
        let line = MINIMAL
            .iter()
            .enumerate()
            .filter(|(j,_)| i != *j)
            .map(|a| *a.1)
            .collect::<Vec<_>>();
        
        if TOAInfo::parse_tempo2(&line).is_ok() {
            panic!("'{}' should fail", line.join(" "));
        }
    }
}
#[test]
fn minimal() {
    TOAInfo::parse_tempo2(MINIMAL).unwrap();
}
#[test]
fn bad() {
    TOAInfo::parse_tempo2(&[&["C"],MINIMAL].concat()).unwrap();
}
#[test]
fn comments() {
    let parts = [
        "#hii",
        "dir/file.ext",
        "999.999",
        "55000.97531",
        "99.11",
        "tele-id",
        "#hellouuu",
    ];
    TOAInfo::parse_tempo2(&parts).unwrap();
}
#[test]
fn file() {
    let file = File::open("test.tim").unwrap();
    let bfr = BufReader::new(file);
    read_tim(bfr).unwrap();
}
