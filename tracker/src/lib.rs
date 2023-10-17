use std::{fmt::Display, string::FromUtf8Error};

use injector::InjectError;

pub mod assets;
pub mod configuration;
pub mod gui;
pub mod logger;
mod logwatcher;
pub mod mtgadb;
mod parser;
pub mod tracker;
mod utils;

#[derive(Debug, Clone)]
pub enum TrackerError {
    IcedError(String),
    SqlError(String),
    SerdeError(String),
    IoError(String),
    InjectError(InjectError),
    ConversionError(String),
    CustomError(String),
    ReqwestError(String),
}

impl From<serde_json::Error> for TrackerError {
    fn from(err: serde_json::Error) -> Self {
        TrackerError::SerdeError(format!("Serde error: {}", err))
    }
}

impl From<std::io::Error> for TrackerError {
    fn from(err: std::io::Error) -> Self {
        TrackerError::IoError(format!("IO error: {}", err))
    }
}

impl From<iced::Error> for TrackerError {
    fn from(err: iced::Error) -> Self {
        TrackerError::IcedError(format!("Iced error: {}", err))
    }
}

impl From<rusqlite::Error> for TrackerError {
    fn from(err: rusqlite::Error) -> Self {
        TrackerError::SqlError(format!("Sql error: {}", err))
    }
}
impl From<&str> for TrackerError {
    fn from(err: &str) -> Self {
        TrackerError::CustomError(String::from(err))
    }
}

impl From<String> for TrackerError {
    fn from(err: String) -> Self {
        TrackerError::CustomError(err)
    }
}

impl From<InjectError> for TrackerError {
    fn from(err: InjectError) -> Self {
        TrackerError::InjectError(err)
    }
}

impl From<FromUtf8Error> for TrackerError {
    fn from(err: FromUtf8Error) -> Self {
        TrackerError::ConversionError(format!("Utf-8 string conversion error: {}", err))
    }
}

impl From<reqwest::Error> for TrackerError {
    fn from(value: reqwest::Error) -> Self {
        TrackerError::ReqwestError(value.to_string())
    }
}

impl Display for TrackerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrackerError::IcedError(e) => write!(f, "{}", e),
            TrackerError::SqlError(e) => write!(f, "{}", e),
            TrackerError::SerdeError(e) => write!(f, "{}", e),
            TrackerError::IoError(e) => write!(f, "{}", e),
            TrackerError::InjectError(e) => write!(f, "{}", e),
            TrackerError::ConversionError(e) => write!(f, "{}", e),
            TrackerError::CustomError(e) => write!(f, "{}", e),
            TrackerError::ReqwestError(e) => write!(f, "{}", e),
        }
    }
}

pub type Result<T> = std::result::Result<T, TrackerError>;
