//! This crate is intended for use with pulsar science projects, mainly to 
//! provide file parsing. 

#![warn(missing_docs)]

pub(crate) mod parse_tools;

pub mod error;
pub mod data_types;
pub mod parfile;
pub mod timfile;
pub mod fits;
