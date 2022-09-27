use std::{error::Error, fmt::Display};

pub mod auth;
pub mod social;

#[derive(Debug)]
struct SqlConversionError {
    message: String,
}

impl Display for SqlConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SqlConversionError")
    }
}

impl Error for SqlConversionError {}
