use std::{error::Error, fmt::Display};

pub mod auth;
pub mod social;
pub mod text;
pub mod wormhole;

mod permashort_link;

pub use crate::permashort_link::*;

#[derive(Debug)]
struct SqlConversionError {
    message: String,
}

impl Display for SqlConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("SqlConversionError: {}", self.message))
    }
}

impl Error for SqlConversionError {}
