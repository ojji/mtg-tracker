use std::{
    env::Args,
    path::{Path, PathBuf},
};

pub struct Config {
    command: TrackerCommand,
}

impl Config {
    pub fn new(args: Args) -> Result<Config, Box<dyn std::error::Error>> {
        let default_collector_path = Path::new(
            r"./mtga-datacollector/bin/x64/Release/netstandard2.1/mtga-datacollector.dll",
        );
        let default_database_path = Path::new(r"./mtga-tracker.db");
        if args.len() == 1 {
            Ok(Config {
                command: TrackerCommand::Parse(ParseParams {
                    collector_dll_path: default_collector_path.to_path_buf(),
                    database_path: default_database_path.to_path_buf(),
                }),
            })
        } else {
            let args: Vec<String> = args.into_iter().collect();
            match args[1].to_lowercase().as_str() {
                "dump" => {
                    let collector_path =
                        args.get(2).map_or(default_collector_path, |p| Path::new(p));
                    Ok(Config {
                        command: TrackerCommand::Dump(collector_path.to_path_buf()),
                    })
                }
                "dumpartists" => {
                    let scryfall_cards_json_path = Path::new(
                        args.get(2)
                            .ok_or("Please provide a scryfall cards db path")?,
                    );

                    let mtga_cards_json_path =
                        Path::new(args.get(3).ok_or("Please provide an mtga cards db path")?);

                    let output_file =
                        Path::new(args.get(3).ok_or("Please provide an output file path")?);

                    Ok(Config {
                        command: TrackerCommand::DumpArtistMapping(DumpArtistMappingParams {
                            scryfall_cards_json_path: scryfall_cards_json_path.to_path_buf(),
                            mtga_cards_json_path: mtga_cards_json_path.to_path_buf(),
                            output_file: output_file.to_path_buf(),
                        }),
                    })
                }
                "createdb" => {
                    let scryfall_cards_json_path = Path::new(
                        args.get(2)
                            .ok_or("Please provide a scryfall cards db path")?,
                    );
                    let mtga_cards_json_path =
                        Path::new(args.get(3).ok_or("Please provide an mtga cards db path")?);

                    let database_path = args.get(4).map_or(default_database_path, |p| Path::new(p));

                    Ok(Config {
                        command: TrackerCommand::CreateDatabase(CreateDatabaseParams {
                            scryfall_cards_json_path: scryfall_cards_json_path.to_path_buf(),
                            mtga_cards_json_path: mtga_cards_json_path.to_path_buf(),
                            database_output_path: database_path.to_path_buf(),
                        }),
                    })
                }
                _ => return Err("Unrecognized command".into()),
            }
        }
    }

    pub fn command(&self) -> &TrackerCommand {
        &self.command
    }
}

pub enum TrackerCommand {
    CreateDatabase(CreateDatabaseParams),
    Parse(ParseParams),
    DumpArtistMapping(DumpArtistMappingParams),
    Dump(PathBuf),
}

pub struct ParseParams {
    pub collector_dll_path: PathBuf,
    pub database_path: PathBuf,
}

pub struct CreateDatabaseParams {
    pub scryfall_cards_json_path: PathBuf,
    pub mtga_cards_json_path: PathBuf,
    pub database_output_path: PathBuf,
}

pub struct DumpArtistMappingParams {
    pub scryfall_cards_json_path: PathBuf,
    pub mtga_cards_json_path: PathBuf,
    pub output_file: PathBuf,
}
