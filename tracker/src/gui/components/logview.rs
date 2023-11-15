use async_std::path::Path;
use iced::{
    widget::{column, container, image, row, scrollable, text, Column},
    Length,
};

use crate::{
    assets,
    gui::{
        components::popup_element::PopupElement, style::container::ContainerStyle, Element,
        TrackerMessage,
    },
    mtgadb::{
        model::InventoryUpdateResult,
        MtgaDb,
    },
};

pub enum LogEntry {
    String(String),
    InventoryUpdate(InventoryUpdateResult),
}

impl LogEntry {
    fn view(&self, database: &MtgaDb) -> Element<TrackerMessage> {
        match self {
            LogEntry::String(content) => text(content).size(12).into(),
            LogEntry::InventoryUpdate(inventory_update) => {
                let time_element: Option<Element<TrackerMessage>> = Some(
                    text(format!("[{}]", inventory_update.friendly_time()))
                        .size(12)
                        .height(30.0)
                        .into(),
                );

                let ticket_element = if let Some(tickets) = inventory_update.tickets() {
                    let total_tickets = tickets.iter().map(|ticket| ticket.count).sum::<i32>();
                    let ticket_elements = tickets
                        .iter()
                        .map(|ticket| {
                            text(format!("{:+} {}", ticket.count, ticket.ticket))
                                .size(12)
                                .into()
                        })
                        .collect::<Vec<Element<TrackerMessage>>>();

                    let ticket_popup =
                        container(Column::with_children(ticket_elements).spacing(3.0))
                            .padding(5.0)
                            .style(ContainerStyle::Box);

                    let ticket_base = row![
                        image(image::Handle::from_memory(assets::TICKET_ICON)).height(30.0),
                        text(format!("{:+}", total_tickets)).size(12)
                    ]
                    .spacing(3.0)
                    .align_items(iced::Alignment::Center);

                    Some(PopupElement::new(ticket_base, ticket_popup).into())
                } else {
                    None
                };

                let gold_element: Option<Element<TrackerMessage>> =
                    if inventory_update.gold_delta() != 0 {
                        Some(
                            row![
                                image(image::Handle::from_memory(assets::GOLD_ICON)).height(30.0),
                                text(format!("{:+}", inventory_update.gold_delta())).size(12)
                            ]
                            .spacing(3.0)
                            .align_items(iced::Alignment::Center)
                            .into(),
                        )
                    } else {
                        None
                    };

                let gem_element: Option<Element<TrackerMessage>> =
                    if inventory_update.gems_delta() != 0 {
                        Some(
                            row![
                                image(image::Handle::from_memory(assets::GEM_ICON)).height(30.0),
                                text(format!("{:+}", inventory_update.gems_delta())).size(12)
                            ]
                            .spacing(3.0)
                            .align_items(iced::Alignment::Center)
                            .into(),
                        )
                    } else {
                        None
                    };

                let xp_element: Option<Element<TrackerMessage>> =
                    if inventory_update.xp_gained() != 0 {
                        Some(
                            row![
                                image(image::Handle::from_memory(assets::XP_ICON)).height(30.0),
                                text(inventory_update.xp_gained()).size(12)
                            ]
                            .spacing(3.0)
                            .align_items(iced::Alignment::Center)
                            .into(),
                        )
                    } else {
                        None
                    };

                let vault_element: Option<Element<TrackerMessage>> =
                    if inventory_update.vault_delta_percent() != 0.0 {
                        Some(
                            row![
                                image(image::Handle::from_memory(assets::VAULT_ICON)).height(30.0),
                                text(format!("{:+}%", inventory_update.vault_delta_percent()))
                            ]
                            .spacing(3.0)
                            .align_items(iced::Alignment::Center)
                            .into(),
                        )
                    } else {
                        None
                    };

                let cards_element: Option<Element<TrackerMessage>> =
                    if !inventory_update.cards_added().is_empty() {
                        let base = row![
                            image(image::Handle::from_memory(assets::SLEEVE_ICON)).height(30.0),
                            text(format!("{:+}", inventory_update.cards_added().len())).size(12)
                        ]
                        .spacing(3.0)
                        .align_items(iced::Alignment::Center);

                        let card_names = inventory_update
                            .cards_added()
                            .into_iter()
                            .map(|arena_id| (arena_id, database.get_tracker_card(arena_id)))
                            .map(|(arena_id, card)| match card {
                                Some(card) => text(card.name()).size(12).into(),
                                None => text(format!("Card #{}", arena_id)).size(12).into(),
                            })
                            .collect::<Vec<Element<TrackerMessage>>>();

                        let popup_cards = container(Column::with_children(card_names).spacing(3.0))
                            .style(ContainerStyle::Box)
                            .padding(3);

                        let aetherized_cards_element = PopupElement::new(base, popup_cards).into();
                        Some(aetherized_cards_element)
                    } else {
                        None
                    };

                let context: Option<Element<TrackerMessage>> = Some(
                    row![
                        text("Source:").size(15),
                        text(inventory_update.source_context()).size(12),
                    ]
                    .spacing(3.0)
                    .align_items(iced::Alignment::Center)
                    .into(),
                );

                let debug_base = image(image::Handle::from_memory(assets::DEBUG_ICON)).height(30.0);
                let json_content = container(
                    text(serde_json::to_string_pretty(inventory_update).unwrap()).size(12),
                )
                .style(ContainerStyle::Box)
                .padding(3.0);
                let debug_element: Option<Element<TrackerMessage>> =
                    Some(PopupElement::new(debug_base, json_content).into());

                let content = iced_aw::Wrap::with_elements(
                    vec![
                        time_element,
                        gold_element,
                        gem_element,
                        xp_element,
                        vault_element,
                        cards_element,
                        ticket_element,
                        context,
                        debug_element,
                    ]
                    .into_iter()
                    .flatten()
                    .collect(),
                )
                .spacing(3.0)
                .align_items(iced::Alignment::Center);

                container(content).into()
            }
        }
    }
}

pub struct LogViewComponent {
    database: MtgaDb,
    messages: Vec<LogEntry>,
}

impl LogViewComponent {
    pub fn new<P>(database_path: P) -> LogViewComponent
    where
        P: AsRef<Path>,
    {
        LogViewComponent {
            messages: vec![],
            database: MtgaDb::new(database_path),
        }
    }

    pub fn add_entry(&mut self, message: LogEntry) {
        self.messages.insert(0, message);
    }

    pub fn view(&self) -> Element<TrackerMessage> {
        let messages: Element<TrackerMessage> = column(
            self.messages
                .iter()
                .map(|msg| msg.view(&self.database))
                .collect::<Vec<Element<_>>>(),
        )
        .width(Length::Fill)
        .into();

        let scrollable_messages = scrollable(messages);

        container(scrollable_messages).into()
    }
}
