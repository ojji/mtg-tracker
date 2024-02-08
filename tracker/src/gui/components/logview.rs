use async_std::path::Path;
use iced::{
    widget::{column, container, image, row, scrollable, text, Column, Row},
    Length,
};

use crate::{
    assets,
    gui::{
        components::popup_element::PopupElement, style::container::ContainerStyle, Element,
        TrackerMessage,
    },
    mtgadb::{model::InventoryUpdateResult, MtgaDb},
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

                let packs_element = if let Some(packs) = inventory_update.packs_delta() {
                    let total_packs = packs.iter().map(|pack| pack.count).sum::<i32>();
                    let pack_elements = packs
                        .iter()
                        .map(|pack| {
                            text(format!("{:+} {}", pack.count, pack.short_name()))
                                .size(12)
                                .into()
                        })
                        .collect::<Vec<Element<TrackerMessage>>>();

                    let packs_popup = container(Column::with_children(pack_elements).spacing(3.0))
                        .padding(5.0)
                        .style(ContainerStyle::Box);

                    let packs_base = row![
                        image(image::Handle::from_memory(assets::PACK_ICON)).height(30.0),
                        text(format!("{:+}", total_packs)).size(12)
                    ]
                    .spacing(3.0)
                    .align_items(iced::Alignment::Center);

                    Some(PopupElement::new(packs_base, packs_popup).into())
                } else {
                    None
                };

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

                let orbs_element: Option<Element<TrackerMessage>> =
                    inventory_update.orbs_delta().map(|orb_delta| {
                        row![
                            image(image::Handle::from_memory(assets::ORB_ICON)).height(30.0),
                            text(format!("{:+}", orb_delta))
                        ]
                        .spacing(3.0)
                        .align_items(iced::Alignment::Center)
                        .into()
                    });

                let draft_token_element: Option<Element<TrackerMessage>> = inventory_update
                    .draft_token_delta()
                    .map(|draft_token_delta| {
                        row![
                            image(image::Handle::from_memory(assets::DRAFT_TOKEN_ICON))
                                .height(30.0),
                            text(format!("{:+}", draft_token_delta))
                        ]
                        .spacing(3.0)
                        .align_items(iced::Alignment::Center)
                        .into()
                    });

                let mythic_wc_element: Option<Element<TrackerMessage>> =
                    if inventory_update.mythic_wildcard_delta() != 0 {
                        Some(
                            row![
                                image(image::Handle::from_memory(assets::MYTHIC_WC_ICON))
                                    .height(30.0),
                                text(format!("{:+}", inventory_update.mythic_wildcard_delta()))
                            ]
                            .spacing(3.0)
                            .align_items(iced::Alignment::Center)
                            .into(),
                        )
                    } else {
                        None
                    };

                let rare_wc_element: Option<Element<TrackerMessage>> =
                    if inventory_update.rare_wildcard_delta() != 0 {
                        Some(
                            row![
                                image(image::Handle::from_memory(assets::RARE_WC_ICON))
                                    .height(30.0),
                                text(format!("{:+}", inventory_update.rare_wildcard_delta()))
                            ]
                            .spacing(3.0)
                            .align_items(iced::Alignment::Center)
                            .into(),
                        )
                    } else {
                        None
                    };

                let uncommon_wc_element: Option<Element<TrackerMessage>> =
                    if inventory_update.uncommon_wildcard_delta() != 0 {
                        Some(
                            row![
                                image(image::Handle::from_memory(assets::UNCOMMON_WC_ICON))
                                    .height(30.0),
                                text(format!("{:+}", inventory_update.uncommon_wildcard_delta()))
                            ]
                            .spacing(3.0)
                            .align_items(iced::Alignment::Center)
                            .into(),
                        )
                    } else {
                        None
                    };

                let common_wc_element: Option<Element<TrackerMessage>> =
                    if inventory_update.common_wildcard_delta() != 0 {
                        Some(
                            row![
                                image(image::Handle::from_memory(assets::COMMON_WC_ICON))
                                    .height(30.0),
                                text(format!("{:+}", inventory_update.common_wildcard_delta()))
                            ]
                            .spacing(3.0)
                            .align_items(iced::Alignment::Center)
                            .into(),
                        )
                    } else {
                        None
                    };

                let vanity_element: Option<Element<TrackerMessage>> = if inventory_update
                    .vanity_items_added()
                    .is_some()
                    || inventory_update.vanity_items_removed().is_some()
                {
                    let mut vanity_item_details: Vec<Element<TrackerMessage>> = vec![];

                    let mut base_items: Vec<Element<TrackerMessage>> =
                        vec![image(image::Handle::from_memory(assets::VANITY_ICON))
                            .height(30.0)
                            .into()];

                    if let Some(vanity_items_added) = inventory_update.vanity_items_added() {
                        base_items.push(
                            text(format!("{:+}", vanity_items_added.len()))
                                .size(12)
                                .into(),
                        );

                        for vanity_item in vanity_items_added {
                            vanity_item_details
                                .push(text(format!("+{}", vanity_item)).size(12).into());
                        }
                    }

                    if let Some(vanity_items_removed) = inventory_update.vanity_items_removed() {
                        base_items.push(
                            text(format!("{:+}", vanity_items_removed.len()))
                                .size(12)
                                .into(),
                        );

                        for vanity_item in vanity_items_removed {
                            vanity_item_details
                                .push(text(format!("-{}", vanity_item)).size(12).into());
                        }
                    }

                    let vanity_items_base = Row::with_children(base_items)
                        .spacing(3.0)
                        .align_items(iced::Alignment::Center);

                    let vanity_items_popup =
                        container(Column::with_children(vanity_item_details).spacing(3.0))
                            .padding(5.0)
                            .style(ContainerStyle::Box);

                    Some(PopupElement::new(vanity_items_base, vanity_items_popup).into())
                } else {
                    None
                };

                let style_element: Option<Element<TrackerMessage>> = if inventory_update
                    .art_skins_added()
                    .is_some()
                    || inventory_update.art_skins_removed().is_some()
                {
                    let mut skins_details: Vec<Element<TrackerMessage>> = vec![];

                    let mut base_items: Vec<Element<TrackerMessage>> =
                        vec![image(image::Handle::from_memory(assets::STYLE_ICON))
                            .height(30.0)
                            .into()];

                    if let Some(skins_added) = inventory_update.art_skins_added() {
                        base_items.push(text(format!("{:+}", skins_added.len())).size(12).into());
                        for skin in skins_added {
                            skins_details
                                .push(text(format!("+{}", skin.logview_name())).size(12).into());
                        }
                    }

                    // is it even possible to trigger a `skin removed` event?
                    if let Some(skins_removed) = inventory_update.art_skins_removed() {
                        base_items.push(text(format!("-{}", skins_removed.len())).size(12).into());
                        for skin in skins_removed {
                            skins_details
                                .push(text(format!("-{}", skin.logview_name())).size(12).into());
                        }
                    }

                    let style_base = Row::with_children(base_items)
                        .spacing(3.0)
                        .align_items(iced::Alignment::Center);

                    let style_popup = container(Column::with_children(skins_details).spacing(3.0))
                        .padding(5.0)
                        .style(ContainerStyle::Box);

                    Some(PopupElement::new(style_base, style_popup).into())
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
                        text("Ctx:").size(15),
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
                        packs_element,
                        vault_element,
                        orbs_element,
                        common_wc_element,
                        uncommon_wc_element,
                        rare_wc_element,
                        mythic_wc_element,
                        vanity_element,
                        style_element,
                        cards_element,
                        ticket_element,
                        draft_token_element,
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
