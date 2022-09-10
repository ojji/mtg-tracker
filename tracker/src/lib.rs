mod configuration;
mod mtgadb;

use std::env::Args;
use injector::Mtga;
use mtgadb::MtgaDb;

pub struct Tracker {
    config: configuration::Config,
}

impl Tracker {
    pub fn new(args: Args) -> Result<Tracker, Box<dyn std::error::Error>> {
        let config = configuration::Config::new(args)?;
        Ok(Tracker { config })
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        match self.config.command() {
            configuration::TrackerCommand::Inject(collector_path) => {
                let injector = Mtga::new()?;
                injector.inject_tracker(collector_path)?;
                println!("Data collector injected successfully.");
                Ok(())
            }
            configuration::TrackerCommand::CreateDatabase(database_path, required_sets) => {
              let database = mtgadb::MtgaDb::new(database_path)?;
              database.create_from_sets(required_sets)?;
              Ok(())
            }
        }
    }
}
