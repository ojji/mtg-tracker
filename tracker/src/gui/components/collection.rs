use async_std::path::Path;
use iced::{Command, Length, widget::{column, text, container, scrollable}};

use crate::{
    gui::{TrackerMessage, Element},
    mtgadb::{MtgaDb, UserSession},
    Result,
};

pub struct CollectionComponent {
    database: MtgaDb,
    selected_set: String,
    display_user_session: Option<UserSession>,
    cards: Vec<String>,
}

impl CollectionComponent {
    pub fn new<P>(database_path: P) -> CollectionComponent
    where
        P: AsRef<Path>,
    {
        CollectionComponent {
            database: MtgaDb::new(database_path),
            selected_set: String::from("ltr"),
            display_user_session: None,
            cards: vec![],
        }
    }

    pub fn set_current_user(&mut self, user: UserSession) -> Result<Command<TrackerMessage>> {
        self.display_user_session = Some(user);
        self.update_state()
    }

    pub fn change_selected_set(&mut self, set: String) -> Result<Command<TrackerMessage>> {
        self.selected_set = set;
        self.update_state()
    }

    fn update_state(&mut self) -> Result<Command<TrackerMessage>> {
        let common_cards = self
            .database
            .get_common_cards_in_boosters(&self.selected_set)?;
        let uncommon_cards = self
            .database
            .get_uncommon_cards_in_boosters(&self.selected_set)?;
        let mythic_cards = self
            .database
            .get_mythic_cards_in_boosters(&self.selected_set)?;
        let rare_cards = self
            .database
            .get_rare_cards_in_boosters(&self.selected_set)?;
        let log_message = format!(
            "There are {} ({}) common, {} ({}) uncommon, {} ({}) rare, {} ({}) mythic cards in {}.",
            common_cards.len(),
            common_cards.len() * 4,
            uncommon_cards.len(),
            uncommon_cards.len() * 4,
            rare_cards.len(),
            rare_cards.len() * 4,
            mythic_cards.len(),
            mythic_cards.len() * 4,
            &self.selected_set
        );

        let user_id = match self.display_user_session.as_ref() {
            Some(user) => user.user_id(),
            None => return Err("Display user has not been set".into()),
        };

        let mut collected_common_cards =
            self.database
                .get_collected_cards_in_boosters(user_id, &self.selected_set, "common")?;
        collected_common_cards.sort_by(|a, b| a.0.name().partial_cmp(b.0.name()).unwrap());

        let mut collected_uncommon_cards = self.database.get_collected_cards_in_boosters(
            user_id,
            &self.selected_set,
            "uncommon",
        )?;
        collected_uncommon_cards.sort_by(|a, b| a.0.name().partial_cmp(b.0.name()).unwrap());

        let mut collected_rare_cards =
            self.database
                .get_collected_cards_in_boosters(user_id, &self.selected_set, "rare")?;
        collected_rare_cards.sort_by(|a, b| a.0.name().partial_cmp(b.0.name()).unwrap());

        let mut collected_mythic_cards =
            self.database
                .get_collected_cards_in_boosters(user_id, &self.selected_set, "mythic")?;
        collected_mythic_cards.sort_by(|a, b| a.0.name().partial_cmp(b.0.name()).unwrap());

        let cards = collected_common_cards
            .iter()
            .chain(collected_uncommon_cards.iter())
            .chain(collected_rare_cards.iter())
            .chain(collected_mythic_cards.iter())
            .map(|c| format!("{}/4 - {}", c.1, c.0.to_string()))
            .collect();

        self.cards = cards;

        Ok(Command::none())
    }

    pub fn view(&self) -> Element<TrackerMessage> {
        let messages: Element<TrackerMessage> = column(
            self.cards
                .iter()
                .map(|msg| text(msg).size(16).into())
                .collect::<Vec<Element<_>>>(),
        )
        .width(Length::Fill)
        .into();

        let messages = container(messages)
            .width(Length::Fill)
            .height(Length::Shrink)
            .padding(20);

        let scrollable_messages = scrollable(messages);

        container(scrollable_messages).into()
    }
}
