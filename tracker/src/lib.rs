pub mod configuration;
pub mod mtgadb;
pub mod tracker;
pub mod gui;
pub mod assets;
mod logwatcher;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;