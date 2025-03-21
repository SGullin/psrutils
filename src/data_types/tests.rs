#[allow(unused)]
use super::*;

#[test]
fn ra_dec_coords() {
    for ra in [
        "RA     59:59.999\n",
        "RA     23:59:\n",
        "RA     24:59:59.999\n",
        "RA     12:60:59.999\n",
        "RA     12:59:60\n",
        "RA     -2:59:59.999\n",
    ] {
        let c = ra.parse::<J2000Ra>();
        if c.is_ok() { 
            panic!("{} should not be ok as ra", ra);
        }
    }

    for ra in [
        "23:59:59.999",
        "0:59:59.999",
        "23:0:59.999",
        "23:59:0.0",
        "0:0:0",
    ] {
        let c = ra.parse::<J2000Ra>();
        if let Err(err) = c { 
            panic!("{}\n{} should be ok as ra", err, ra);
        }
    }

    for dec in [    
        "59:59.999",
        "0:59:",
        "90:59:59.999",
        "90:0:0.001",
        "0:60:59.999",
        "0:59:60",
        "-90:59:59.999",
        "-90:0:0.001",
    ] {
        let c = dec.parse::<J2000Dec>();
        if c.is_ok() { 
            panic!("{} should not be ok as dec", dec);
        }
    }

    for dec in [
        "89:59:59.999",
        "90:0:0.0",
        "0:59:59.999",
        "89:0:59.999",
        "89:59:0.0",
        "0:0:0",
        "-0:0:0",
        "-89:59:59.999",
        "-90:0:0.0",
    ] {
        let c = dec.parse::<J2000Dec>();
        if let Err(err) = c {
            panic!("{}\n{} should be ok as dec", err, dec);
        }
    }
}
