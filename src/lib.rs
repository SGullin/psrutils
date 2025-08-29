//! This crate is intended for use with pulsar science projects, mainly to 
//! provide file parsing. 
#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
)]
#![allow(clippy::must_use_candidate)]

pub(crate) mod parse_tools;

pub mod error;
pub mod data_types;
pub mod parfile;
pub mod timfile;
