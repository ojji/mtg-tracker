#![windows_subsystem = "windows"]

use std::env;
use tracker::{configuration::Config, tracker::Tracker};

use async_std::task;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn async_main() -> Result<()> {
    let config = Config::new(env::args())?;
    let mut tracker = Tracker::new();
    tracker.run(config).await?;
    Ok(())
}

fn main() -> Result<()> {
    let tracker_task = async_main();
    task::block_on(tracker_task)
}
