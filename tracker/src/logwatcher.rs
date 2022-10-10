use crate::mtgadb::model::{self, AccountInfoEvent, CollectionEvent, InventoryUpdateEvent, InventoryEvent};
use crate::Result;
use async_std::fs::File;
use async_std::io::prelude::SeekExt;
use async_std::io::ReadExt;
use async_std::path::{Path, PathBuf};
use std::io::SeekFrom;

#[derive(Debug)]
pub enum LineParseResult {
    None,
    UserSession(AccountInfoEvent),
    CollectionEvent(CollectionEvent),
    InventoryUpdateEvent(InventoryUpdateEvent),
    InventoryEvent(InventoryEvent),
}

#[derive(Debug, Clone)]
pub struct LogWatcher {
    path: PathBuf,
    last_file_size: u64,
}

impl LogWatcher {
    pub fn new(path: PathBuf) -> LogWatcher {
        LogWatcher {
            path,
            last_file_size: 0,
        }
    }

    pub async fn read_log(&mut self) -> Result<Vec<u8>> {
        let mut f = File::open(self.path.as_path()).await?;
        let metadata = f.metadata().await?;
        let current_file_size = metadata.len();

        let mut cursor_pos = 0;
        let mut bytes_to_read = current_file_size;
        if current_file_size != self.last_file_size {
            if current_file_size > self.last_file_size {
                cursor_pos = self.last_file_size;
                bytes_to_read = current_file_size - self.last_file_size;
            }

            let mut buf = vec![0_u8; bytes_to_read as usize];
            f.seek(SeekFrom::Start(cursor_pos)).await?;
            f.read_exact(&mut buf).await?;
            self.last_file_size = current_file_size;
            Ok(buf)
        } else {
            Ok(vec![])
        }
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }
}

pub async fn watch_log(watcher: LogWatcher) -> Result<(LogWatcher, Vec<LineParseResult>)> {
    let mut watcher = watcher;
    let content_read = String::from_utf8(watcher.read_log().await?)?;

    let mut lines_read = vec![];
    for line in content_read
        .lines()
        .filter(|&line| line.starts_with("[MTGADataCollector]"))
    {
        lines_read.push(parse_line(line).await?);
    }

    Ok((watcher, lines_read))
}

pub async fn parse_line(line: &str) -> Result<LineParseResult> {
    const COLLECTION_PREFIX: &str = "[MTGADataCollector][collection]";
    const INVENTORY_PREFIX: &str = "[MTGADataCollector][inventory]";
    const INVENTORY_UPDATE_PREFIX: &str = "[MTGADataCollector][inventory-update]";
    const ACCOUNT_INFO_PREFIX: &str = "[MTGADataCollector][account-info]";

    if line.starts_with(ACCOUNT_INFO_PREFIX) {
        let account_info: AccountInfoEvent =
            serde_json::from_str(&line[ACCOUNT_INFO_PREFIX.len()..])?;
        return Ok(LineParseResult::UserSession(account_info));
    } else if line.starts_with(COLLECTION_PREFIX) {
        let collection: CollectionEvent = serde_json::from_str(&line[COLLECTION_PREFIX.len()..])?;
        return Ok(LineParseResult::CollectionEvent(collection));
    } else if line.starts_with(INVENTORY_PREFIX) {
        let inventory: model::InventoryEvent =
            serde_json::from_str(&line[INVENTORY_PREFIX.len()..])?;
        return Ok(LineParseResult::InventoryEvent(inventory));
    } else if line.starts_with(INVENTORY_UPDATE_PREFIX) {
        let inventory_update: model::InventoryUpdateEvent =
            serde_json::from_str(&line[INVENTORY_UPDATE_PREFIX.len()..])?;
        return Ok(LineParseResult::InventoryUpdateEvent(inventory_update));
    }

    Ok(LineParseResult::None)
}
