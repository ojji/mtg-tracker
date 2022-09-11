mod configuration;
mod mtgadb;

use injector::Mtga;
use mtgadb::MtgaDb;
use std::env::Args;

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
                let start = std::time::Instant::now();
                let database = MtgaDb::create_from_scryfall_db(database_path, &required_sets)?;
                let elapsed = start.elapsed();
                println!(
                    "[{:.2?}] Scryfall database read from: {}, creating card-db for the tracker... ",
                    elapsed,
                    database_path.to_str().unwrap()
                );

                let start = std::time::Instant::now();
                database.export(self.config.database_path())?;
                let elapsed = start.elapsed();

                println!(
                    "[{:.2?}] Card-db has been written into {}.",
                    elapsed,
                    self.config.database_path().to_str().unwrap()
                );
                Ok(())
            }
        }
    }
}
