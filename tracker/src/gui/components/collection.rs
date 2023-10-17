use async_std::path::Path;
use iced::{
    widget::{column, container, image, row, scrollable, text, Column},
    Command, Length,
};

use crate::{
    gui::{
        components::popup_element::PopupElement, style::container::ContainerStyle, Element,
        TrackerMessage,
    },
    mtgadb::{MtgaDb, UserSession},
    utils::ImageLoader,
    Result,
};

pub struct CollectionCard {
    arena_id: u32,
    long_name: String,
    name: String,
    image: Option<image::Handle>,
    max_collected: u32,
    num_collected: u32,
    ratings_labels: Vec<String>,
    ratings_values: Vec<String>,
    tags: Vec<String>,
    notes: String,
    image_loaded: bool,
}

pub struct CollectionModel {
    cards: Vec<CollectionCard>,
}

pub struct CollectionComponent {
    database: MtgaDb,
    selected_set: String,
    display_user_session: Option<UserSession>,
    model: CollectionModel,
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
            model: CollectionModel { cards: vec![] },
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
            .chain(collected_uncommon_cards)
            .chain(collected_rare_cards)
            .chain(collected_mythic_cards)
            .map(|(tracker_card, collected)| CollectionCard {
                arena_id: tracker_card.arena_id(),
                long_name: tracker_card.to_string(),
                name: tracker_card.name().to_string(),
                image: None,
                max_collected: tracker_card.max_collected(),
                num_collected: collected,
                ratings_labels: vec![],
                ratings_values: vec![],
                tags: vec![],
                notes: String::from("notes"),
                image_loaded: false,
            })
            .collect();

        self.model.cards = cards;

        Ok(Command::none())
    }

    pub fn view(&self) -> Element<TrackerMessage> {
        let card_rows: Element<TrackerMessage> = column(
            self.model
                .cards
                .iter()
                .map(|c| card_row_container(c))
                .collect::<Vec<Element<_>>>(),
        )
        .width(Length::Fill)
        .into();

        let messages = container(card_rows)
            .width(Length::Fill)
            .height(Length::Shrink)
            .padding(20);

        let scrollable_messages = scrollable(messages);

        container(scrollable_messages).into()
    }

    pub fn hover_card(&mut self, arena_id: u32) -> Command<TrackerMessage> {
        let card_is_loaded = self
            .model
            .cards
            .iter()
            .find(|&card| card.arena_id == arena_id)
            .map(|card| card.image_loaded);

        if let Some(false) = card_is_loaded {
            Command::perform(
                ImageLoader::load_image(arena_id, self.database.clone()),
                TrackerMessage::CardImageLoaded,
            )
        } else {
            Command::none()
        }
    }

    pub fn image_loaded(
        &mut self,
        arena_id: u32,
        image_handle: iced_futures::core::image::Handle,
    ) -> Command<TrackerMessage> {
        let card_to_update = self
            .model
            .cards
            .iter_mut()
            .find(|card| card.arena_id == arena_id);

        if let Some(card) = card_to_update {
            card.image = Some(image_handle);
            card.image_loaded = true;
        }

        Command::none()
    }
}

fn card_row_container(card: &CollectionCard) -> Element<TrackerMessage> {
    let base = text(format!(
        "{}/{} {}",
        card.num_collected, card.max_collected, card.long_name
    ))
    .size(16);

    let card_name = container(text(card.name.as_str()).size(16))
        .style(ContainerStyle::Box)
        .padding(5);

    let image_content: Element<TrackerMessage> = {
        if card.image_loaded {
            if let Some(image_handle) = card.image.as_ref() {
                container(image(image_handle.clone()))
                    .width(269.0)
                    .height(375.0)
                    .into()
            } else {
                container(image(image::Handle::from_path(
                    "assets/cards/no_image_available.png",
                )))
                .width(269.0)
                .height(375.0)
                .into()
            }
        } else {
            container(text("One moment please...").size(16))
                .style(ContainerStyle::Box)
                .into()
        }
    };

    let collection_and_ratings: Element<TrackerMessage> = {
        let children = vec![container(
            text(format!(
                "Collection: {}/{}",
                card.num_collected, card.max_collected
            ))
            .size(16),
        )
        .style(ContainerStyle::Box)
        .padding(5)
        .into()]
        .into_iter()
        .chain(
            card.ratings_labels
                .iter()
                .zip(card.ratings_values.iter())
                .map(|(rating_label, rating_value)| {
                    container(text(format!("{}: {}", rating_label, rating_value)).size(16))
                        .padding(5)
                        .style(ContainerStyle::Box)
                        .into()
                }),
        )
        .chain(
            card.tags
                .iter()
                .map(|tag| {
                    container(text(tag).size(16))
                        .padding(5)
                        .style(ContainerStyle::Box)
                        .into()
                })
                .collect::<Vec<Element<TrackerMessage>>>(),
        )
        .collect::<Vec<Element<TrackerMessage>>>();

        iced_aw::helpers::wrap_horizontal(children)
            .spacing(3.0)
            .line_spacing(3.0)
            .into()
    };

    let content = vec![
        row![image_content].into(),
        row![card_name].into(),
        row![collection_and_ratings].into(),
        row![container(text(format!("Notes: {}", card.notes)).size(16))
            .padding(5)
            .style(ContainerStyle::Box)]
        .into(),
    ]
    .into_iter()
    .collect::<Vec<Element<TrackerMessage>>>();

    let popup = Column::with_children(content).spacing(3);

    PopupElement::new(base, popup)
        .on_hovered(TrackerMessage::Action(crate::gui::Action::CardHovered(
            card.arena_id,
        )))
        .into()
}
