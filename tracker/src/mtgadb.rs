use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;
mod model;

use rusqlite::{params, Connection};
use serde_json;

use model::ScryCard;

pub struct MtgaDb {
    arena_cards: HashMap<u32, ScryCard>,
}

impl MtgaDb {
    pub fn create_from_scryfall_db<P>(
        scryfall_db_path: P,
        required_sets: &[&str],
    ) -> Result<MtgaDb, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let mut arena_cards = HashMap::new();

        let required_sets = required_sets
            .into_iter()
            .map(|&set| set.to_lowercase())
            .collect::<Vec<String>>();

        let mut file = File::open(scryfall_db_path)?;

        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        let result: Vec<ScryCard> = serde_json::from_slice(&data)?;

        for card in result {
            if required_sets.iter().any(|s| *s == card.set) && card.arena_id.is_some() {
                arena_cards.entry(card.arena_id.unwrap()).or_insert(card);
            }
        }
        Ok(MtgaDb { arena_cards })
    }

    pub fn arena_cards(&self) -> &HashMap<u32, ScryCard> {
        &self.arena_cards
    }

    pub fn export<P>(&self, path: P) -> Result<(), Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let mut db = Connection::open(path)?;
        let tx = db.transaction()?;

        tx.execute("DROP TABLE IF EXISTS cards_db", [])?;
        tx.execute(
            "CREATE TABLE IF NOT EXISTS cards_db (
            'arena_id' INTEGER PRIMARY KEY NOT NULL,
            'set' TEXT NOT NULL,
            'rarity' TEXT NOT NULL,
            'booster' INTEGER NOT NULL,
            'data' BLOB NOT NULL
        )",
            [],
        )?;

        tx.execute("CREATE INDEX set_idx ON cards_db('set')", [])?;
        tx.execute("CREATE INDEX rarity_idx ON cards_db('rarity')", [])?;

        for (arena_id, card) in self.arena_cards() {
            tx.execute("INSERT INTO cards_db ('arena_id', 'set', 'rarity', 'booster', 'data') VALUES (?1, ?2, ?3, ?4, ?5)",
            params![arena_id, card.set, card.rarity, card.booster as i32, serde_json::to_value(card)?])?;
        }

        tx.commit()?;
        Ok(())
    }
}
