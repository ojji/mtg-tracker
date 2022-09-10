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
        if args.len() == 1 {
            Ok(Config {
                command: TrackerCommand::Inject(default_collector_path.to_path_buf()),
            })
        } else {
            let args: Vec<String> = args.into_iter().collect();
            match args[1].to_lowercase().as_str() {
                "inject" => {
                    let collector_path =
                        args.get(2).map_or(default_collector_path, |p| Path::new(p));
                    Ok(Config {
                        command: TrackerCommand::Inject(collector_path.to_path_buf()),
                    })
                }
                "createdb" => {
                    let database_path =
                        Path::new(args.get(2).ok_or("Please provide a database path")?);

                    let required_sets = "MIR,WTH,MMQ,INV,PLS,ODY,TOR,JUD,ONS,LGN,SCG,8ED,MRD,DST,5DN,CHK,SOK,9ED,RAV,DIS,CSP,TSP,10E,LRW,MOR,SHM,ME2,ALA,ARB,M10,ZEN,WWK,ROE,M11,SOM,ME4,MBS,NPH,CMD,M12,ISD,DKA,AVR,CONF,M13,PLC,RTR,GTC,DGM,M14,THS,BNG,JOU,C13,MMA,M15,KTK,FRF,VMA,DTK,ORI,BFZ,OGW,SOI,EMN,AER,AKH,HOU,XLN,RIX,DAR,M19,ArenaSUP,G18,GRN,ANA,RNA,WAR,M20,ELD,THB,IKO,MH1,M21,JMP,UND,ZNR,C20,SLD,AKR,UST,KHM,ANB,KLR,MH2,STX,AFR,STA,CMR,2XM,MID,VOW,J21,NEO,Y22,SNC,HBG,DMU,C18,C21,CC2,NEC,UMA,WC".split(',').collect::<Vec<&str>>();

                    Ok(Config {
                        command: TrackerCommand::CreateDatabase(
                            database_path.to_path_buf(),
                            required_sets,
                        ),
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
    Inject(PathBuf),
    CreateDatabase(PathBuf, Vec<&'static str>),
}
