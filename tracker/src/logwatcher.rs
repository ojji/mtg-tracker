use crate::Result;
use async_std::fs::File;
use async_std::io::prelude::SeekExt;
use async_std::io::ReadExt;
use async_std::path::{Path, PathBuf};
use std::io::SeekFrom;

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
