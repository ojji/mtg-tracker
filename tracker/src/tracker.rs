use injector::Injector;

use crate::configuration::{Config, TrackerCommand};
use crate::gui;
use crate::mtgadb::MtgaDb;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub struct Tracker {}

impl Tracker {
    pub fn new() -> Tracker {
        Tracker {}
    }

    pub async fn run(&mut self, config: Config) -> Result<()> {
        match config.command() {
            TrackerCommand::CreateDatabase(params) => {
                let start = std::time::Instant::now();
                MtgaDb::create_database(
                    params.scryfall_cards_json_path.as_path(),
                    params.mtga_cards_json_path.as_path(),
                    params.database_output_path.as_path(),
                )
                .await?;
                let elapsed = start.elapsed();
                println!("[{:.2?}] Database created.", elapsed);
                Ok(())
            }
            TrackerCommand::Parse(params) => {
                gui::run(params)?;
                Ok(())
            }
            TrackerCommand::DumpArtistMapping(params) => {
                MtgaDb::dump_artist_mapping_errors(
                    params.scryfall_cards_json_path.as_path(),
                    params.mtga_cards_json_path.as_path(),
                    params.output_file.as_path(),
                )
                .await?;
                Ok(())
            }
            TrackerCommand::Dump(collector_path) => {
                let injector = Injector::new().await?;
                injector.inject_dumper(collector_path).await?;
                println!("Data dumper injected successfully.");
                Ok(())
            }
        }
    }
}
