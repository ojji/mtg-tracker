use async_std::path::Path;
use iced::{
    widget::{column, container, scrollable, text},
    Command, Length,
};

use crate::{
    gui::{Element, TrackerMessage},
    mtgadb::{model::TrackerCard, MtgaDb, UserSession},
    Result,
};

pub struct CollectionComponent {
    database: MtgaDb,
    selected_set: String,
    display_user_session: Option<UserSession>,
    cards: Vec<(TrackerCard, u32)>,
}

impl CollectionComponent {
    pub fn new<P>(database_path: P) -> CollectionComponent
    where
        P: AsRef<Path>,
    {
        CollectionComponent {
            database: MtgaDb::new(database_path),
            selected_set: String::from("woe"),
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
            .into_iter()
            .chain(collected_uncommon_cards.into_iter())
            .chain(collected_rare_cards.into_iter())
            .chain(collected_mythic_cards.into_iter())
            .collect();

        self.cards = cards;

        Ok(Command::none())
    }

    pub fn view(&self) -> Element<TrackerMessage> {
        let messages: Element<TrackerMessage> = column(
            self.cards
                .iter()
                .map(|c| card_row_container(&c.0, c.1))
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

fn card_row_container(card: &TrackerCard, number_collected: u32) -> Element<TrackerMessage> {
    text(format!(
        "{}/{} {}",
        number_collected,
        card.max_collected(),
        card.to_string()
    ))
    .size(16)
    .into()
}
