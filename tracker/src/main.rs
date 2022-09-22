#![windows_subsystem = "windows"]

use std::env;
use tracker::{configuration::Config, tracker::Tracker};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new(env::args())?;
    let mut tracker = Tracker::new();
    tracker.run(config)?;
    Ok(())
}
