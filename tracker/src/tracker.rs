use injector::Mtga;

use crate::configuration::{Config, TrackerCommand};
use crate::mtgadb::model::{AccountInfoData, AccountInfoEvent, CollectionEvent};
use crate::mtgadb::{MtgaDb, UserSession};

use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::JoinHandle;
use std::time::Duration;

pub struct Tracker {
    user_session: Option<UserSession>,
}

impl Tracker {
    pub fn new() -> Tracker {
        Tracker { user_session: None }
    }

    pub fn run(&mut self, config: Config) -> Result<(), Box<dyn Error>> {
        match config.command() {
            TrackerCommand::CreateDatabase(params) => self.create_database(
                &params.scryfall_cards_json_path,
                &params.mtga_cards_json_path,
                &params.database_output_path,
            ),
            TrackerCommand::Parse(params) => {
                self.parse(&params.collector_dll_path, &params.database_path)
            }
            TrackerCommand::DumpArtistMapping(params) => MtgaDb::dump_artist_mapping_errors(
                &params.scryfall_cards_json_path,
                &params.mtga_cards_json_path,
                &params.output_file,
            ),
            TrackerCommand::Dump(collector_path) => self.dump(collector_path),
        }
    }

    fn dump(&self, collector_dll_path: &PathBuf) -> Result<(), Box<dyn Error>> {
        let injector = Mtga::new()?;
        injector.inject_dumper(collector_dll_path)?;
        println!("Data dumper injected successfully.");
        Ok(())
    }

    fn create_database(
        &self,
        scryfall_cards_json_path: &PathBuf,
        mtga_cards_json_path: &PathBuf,
        database_path: &PathBuf,
    ) -> Result<(), Box<dyn Error>> {
        let start = std::time::Instant::now();
        MtgaDb::create_database(
            scryfall_cards_json_path,
            mtga_cards_json_path,
            database_path,
        )?;
        let elapsed = start.elapsed();
        println!("[{:.2?}] Database created.", elapsed);
        Ok(())
    }

    fn parse<P>(&mut self, collector_dll_path: P, database_path: P) -> Result<(), Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let injector = Mtga::new()?;
        injector.inject_tracker(collector_dll_path)?;
        println!("Data collector injected successfully.");

        let db = MtgaDb::new(database_path)?;

        let log_path: PathBuf = if cfg!(windows) {
            [
                env::var("APPDATA")?.as_str(),
                "..",
                "LocalLow",
                "Wizards Of The Coast",
                "MTGA",
                "Player.log",
            ]
            .iter()
            .collect()
        } else {
            todo!()
        };

        let mut watcher = LogWatcher::new(log_path);

        for event in watcher.events() {
            match *event {
                Ok(v) => {
                    for line in v
                        .lines()
                        .filter(|&line| line.starts_with("[MTGADataCollector]"))
                    {
                        self.parse_line(&db, line)?;
                    }
                }
                Err(_) => todo!(),
            }
        }

        watcher.wait_for_finish();

        Ok(())
    }

    fn parse_line(&mut self, database: &MtgaDb, line: &str) -> Result<(), Box<dyn Error>> {
        const COLLECTION_PREFIX: &str = "[MTGADataCollector][collection]";
        const INVENTORY_PREFIX: &str = "[MTGADataCollector][inventory]";
        const INVENTORY_UPDATE_PREFIX: &str = "[MTGADataCollector][inventory-update]";
        const ACCOUNT_INFO_PREFIX: &str = "[MTGADataCollector][account-info]";

        if line.starts_with(ACCOUNT_INFO_PREFIX) {
            let account_info: AccountInfoEvent =
                serde_json::from_str(&line[ACCOUNT_INFO_PREFIX.len()..])?;
            println!("Account_info update at: {}", account_info.timestamp);
            self.update_user_session(database, account_info.attachment)?;
            println!(
                "Current user: {}",
                self.user_session
                    .as_ref()
                    .ok_or("No user session is set")?
                    .screen_name()
            )
        } else if line.starts_with(COLLECTION_PREFIX) {
            let collection: CollectionEvent =
                serde_json::from_str(&line[COLLECTION_PREFIX.len()..])?;
            println!("Collection update at: {}", collection.timestamp);
            let current_user = self.user_session.as_ref().ok_or("No user session is set")?;
            self.update_user_collection(database, current_user, collection)?;
        } else if line.starts_with(INVENTORY_PREFIX) {
        } else if line.starts_with(INVENTORY_UPDATE_PREFIX) {
        }
        Ok(())
    }

    fn update_user_session(
        &mut self,
        database: &MtgaDb,
        account_info: AccountInfoData,
    ) -> Result<(), Box<dyn Error>> {
        self.user_session = Some(database.get_user_session(account_info)?);
        Ok(())
    }

    fn update_user_collection(
        &self,
        database: &MtgaDb,
        current_user: &UserSession,
        collection: CollectionEvent,
    ) -> Result<(), Box<dyn Error>> {
        database.update_user_collection(current_user, collection)?;
        Ok(())
    }
}

struct LogWatcher {
    events: Receiver<Box<Result<String, Box<dyn Error + Send + Sync + 'static>>>>,
    handle: Option<JoinHandle<()>>,
}

impl LogWatcher {
    pub fn new(file_to_watch: PathBuf) -> LogWatcher {
        // TODO: this is a mess, fix it
        let (tx, rx): (
            Sender<Box<Result<String, Box<dyn Error + Send + Sync + 'static>>>>,
            Receiver<Box<Result<String, Box<dyn Error + Send + Sync + 'static>>>>,
        ) = channel();

        let handle = std::thread::spawn(move || {
            let file_to_watch = file_to_watch.as_path();
            let mut previous_file_size = 0;

            loop {
                std::thread::sleep(Duration::from_secs(1));

                let mut f = File::open(file_to_watch).unwrap();
                let metadata = f.metadata().unwrap();
                let current_file_size = metadata.len();

                let mut cursor_pos = 0;
                let mut bytes_to_read = current_file_size;
                if current_file_size != previous_file_size {
                    if current_file_size > previous_file_size {
                        cursor_pos = previous_file_size;
                        bytes_to_read = current_file_size - previous_file_size;
                    }

                    let mut buf = vec![0_u8; bytes_to_read as usize];

                    f.seek(SeekFrom::Start(cursor_pos)).unwrap();
                    f.read_exact(&mut buf).unwrap();

                    if let Err(_) = tx.send(Ok(String::from_utf8(buf).unwrap()).into()) {
                        break;
                    };

                    previous_file_size = current_file_size;
                }
            }
        });

        LogWatcher {
            events: rx,
            handle: Some(handle),
        }
    }

    fn events(&self) -> &Receiver<Box<Result<String, Box<dyn Error + Send + Sync + 'static>>>> {
        &self.events
    }

    fn wait_for_finish(&mut self) {
        self.handle.take().map(JoinHandle::join);
    }
}
