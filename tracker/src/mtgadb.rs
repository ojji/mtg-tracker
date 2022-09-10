use std::error::Error;
use std::path::Path;

pub(crate) struct MtgaDb {}

impl MtgaDb {
    pub(crate) fn new<P>(p: P) -> Result<MtgaDb, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        todo!()
    }

    pub(crate) fn create_from_sets(&self, required_sets: &[&str]) -> Result<(), Box<dyn Error>> {
        todo!()
    }
}
