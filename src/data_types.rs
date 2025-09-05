//! Contains useful datatypes.

mod j2000;
mod mjd;
mod tests;

pub use j2000::{DECCoordType, J2000Coord, J2000Dec, J2000Ra, RACoordType};
pub use mjd::Mjd;
