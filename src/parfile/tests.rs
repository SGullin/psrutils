#[allow(unused)]
use std::io::{BufReader, LineWriter};
#[allow(unused)]
use super::*;

#[test]
fn incomplete_parinfo() {
    let minimal = [
        "RA 23:59:59.999\n",
        "DEC 359:59:59.999\n",
        "PSR J0000-9999\n",
        "PEPOCH 55000\n",
        "F0 9001\n",
        "DM 99\n",
    ];

    for i in 2..6 {
        let subset = minimal
            .iter()
            .enumerate()
            .fold(String::new(), |a, (j, line)| 
                if i==j { a }
                else { a + line }
            );
        
        let par = Parfile::read(BufReader::new(subset.as_bytes()));
        assert!(par.is_err());
    }
}

#[test]
fn minimal_parinfo() {
    let minimal = "
        PSR    J0000-9999\n\
        RA     23:59:59.999\n\
        DEC    45:59:59.999\n\
        PEPOCH 55000\n\
        F0     9001\n\
        DM 1001.1\n\
    ";

    let par = Parfile::read(BufReader::new(minimal.as_bytes()));
    match par {
        Ok(_) => {},
        Err(err) => panic!("{}", err),
    }
}

#[test]
fn ra_dec_coords() {
    let minimal = "
        PSR    J0000-9999\n\
        PEPOCH 55000\n\
        F0     9001\n\
        DM     1001.1\n\
    ";

    for ra in [
        "RA     59:59.999\n",
        "RA     23:59:\n",
        "RA     24:59:59.999\n",
        "RA     12:60:59.999\n",
        "RA     12:59:60\n",
        "RA     -2:59:59.999\n",
    ] {
        let l = minimal.to_string() + "DEC    89:59:59.999\n" + ra;
        let par = Parfile::read(BufReader::new(l.as_bytes()));
        if par.is_ok() { 
            panic!("{} should not be ok", ra);
        }
    }

    for ra in [
        "RA     23:59:59.999\n",
        "RA     0:59:59.999\n",
        "RA     23:0:59.999\n",
        "RA     23:59:0.0\n",
        "RA     0:0:0\n",
    ] {
        let l = minimal.to_string() + "DEC    89:59:59.999\n" + ra;
        let par = Parfile::read(BufReader::new(l.as_bytes()));
        if let Err(err) = par { 
            panic!("{}\n{} should be ok", err, ra);
        }
    }

    for dec in [    
        "DEC     59:59.999\n",
        "DEC     0:59:\n",
        "DEC     90:59:59.999\n",
        "DEC     90:0:0.001\n",
        "DEC     0:60:59.999\n",
        "DEC     0:59:60\n",
        "DEC     -90:59:59.999\n",
        "DEC     -90:0:0.001\n",
    ] {
        let l = minimal.to_string() + "RA     23:59:59.999\n" + dec;
        let par = Parfile::read(BufReader::new(l.as_bytes()));
        if par.is_ok() { 
            panic!("{} should not be ok", dec);
        }
    }

    for dec in [
        "DEC     89:59:59.999\n",
        "DEC     90:0:0.0\n",
        "DEC     0:59:59.999\n",
        "DEC     89:0:59.999\n",
        "DEC     89:59:0.0\n",
        "DEC     0:0:0\n",
        "DEC     -0:0:0\n",
        "DEC     -89:59:59.999\n",
        "DEC     -90:0:0.0\n",
    ] {
        let l = minimal.to_string() + "RA     23:59:59.999\n" + dec;
        let par = Parfile::read(BufReader::new(l.as_bytes()));
        if let Err(err) = par { 
            panic!("{}\n{} should be ok", err, dec);
        }
    }
}

#[test]
fn duplicate_params() {
    // Text param
    let lines = "
        PSR J0000-9999\n\
        RA 23:59:59.999\n\
        DEC 359:59:59.999\n\
        PEPOCH 55000\n\
        F0 9001\n\
        PSR J3164-9999\n\
        DM 0.1\n\
    ";

    let par = Parfile::read(BufReader::new(lines.as_bytes()));
    assert!(par.is_err());

    // f64 param
    let lines = "
        PSR J0000-9999\n\
        RA 23:59:59.999\n\
        DEC 359:59:59.999\n\
        PEPOCH 55000\n\
        F0 9001\n\
        F1 0.0002\n\
        F1 0.002\n\
        DM 99.99\n\
        F1 0.002\n\
    ";

    let par = Parfile::read(BufReader::new(lines.as_bytes()));
    assert!(par.is_err());

    // Flag
    let lines = "
        PSR J0000-9999\n\
        RA 23:59:59.999\n\
        DEC 359:59:59.999\n\
        IMP\n\
        PEPOCH 55000\n\
        F0 9001\n\
        DM 99.99\n\
        IMP\n\
    ";

    let par = Parfile::read(BufReader::new(lines.as_bytes()));
    assert!(par.is_err());
}

#[test]
fn write_read_invariance() {
    let minimal = "
        PSR     J0000-9999\n\
        RA      23:59:59.999\n\
        DEC     45:59:59.999\n\
        PEPOCH  55000\n\
        F0      9001\n\
        DM      1001.1\n\
        F1      -0.02\n\
        MODE 1\n\
        MODEL   BT\n\
        NOTRACK y\n\
        NITS    1000\n\
        DM_SERIES   TAYLOR\n\
    ";

    let par = Parfile::read(BufReader::new(minimal.as_bytes())).unwrap();
    let mut writer = LineWriter::new(Vec::new());
    par.write(&mut writer).unwrap();
    let src = writer.into_inner().unwrap();

    let par = Parfile::read(BufReader::new(src.as_slice())).unwrap();
    let mut writer = LineWriter::new(Vec::new());
    par.write(&mut writer).unwrap();
    let dst = writer.into_inner().unwrap();

    assert_eq!(src, dst)
}
