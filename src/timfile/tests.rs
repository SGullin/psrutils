#[allow(unused)]
use super::*;
#[allow(unused)]
use std::io::{BufReader, LineWriter};

#[allow(unused)]
const T2_MINIMAL: &[&str] =
    &["dir/file.ext", "999.999", "55000.97531", "99.11", "tele-id"];

#[test]
fn t2_incomplete() {
    for i in 0..T2_MINIMAL.len() {
        let line = T2_MINIMAL
            .iter()
            .enumerate()
            .filter(|(j, _)| i != *j)
            .map(|a| *a.1)
            .collect::<Vec<_>>();

        assert!(
            TOAInfo::parse_tempo2(&line).is_err(),
            "'{}' should fail",
            line.join(" "),
        );
    }
}
#[test]
fn t2_minimal() {
    TOAInfo::parse_tempo2(T2_MINIMAL).unwrap();
}
#[test]
fn t2_bad() {
    TOAInfo::parse_tempo2(&[&["C"], T2_MINIMAL].concat()).unwrap();
}
#[test]
fn t2_comments() {
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
// #[test]
// fn file() {
//     read_tim("testing/test.tim".into(), TimFormat::Tempo2).unwrap();
// }
