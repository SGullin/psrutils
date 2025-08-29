//! Contains useful datatypes.

mod j2000;
mod mjd;
mod tests;

pub use j2000::{
    J2000Coord,
    J2000Ra,
    J2000Dec,
    RACoordType,
    DECCoordType,
};
pub use mjd::Mjd;
