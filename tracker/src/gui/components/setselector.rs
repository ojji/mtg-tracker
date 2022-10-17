use iced::{
    pure::{button, container, image, row, tooltip, Element},
    Alignment, Command, Length,
};

use crate::{
    assets::*,
    gui::{style, Action, TrackerMessage},
};

pub struct SetSelectorComponent {
    set_selected: String,
}

#[derive(Debug, Clone)]
pub enum SetSelectorMessage {
    Interaction(Interaction),
}

#[derive(Debug, Clone)]
pub enum Interaction {
    SetSelected(String),
}

impl SetSelectorComponent {
    pub fn new() -> SetSelectorComponent {
        SetSelectorComponent {
            set_selected: String::from("dmu"),
        }
    }

    pub fn view(&self) -> Element<TrackerMessage> {
        let sets = vec![
            (
                "dmu",
                DMU_SYMBOL,
                Length::Units(32),
                "Dominaria United - DMU",
            ),
            (
                "hbg",
                HBG_SYMBOL,
                Length::Units(32),
                "Alchemy Horizons: Baldur's Gate - HBG",
            ),
            (
                "snc",
                SNC_SYMBOL,
                Length::Units(28),
                "Streets of New Capenna - SNC",
            ),
            (
                "neo",
                NEO_SYMBOL,
                Length::Units(28),
                "Kamigawa: Neon Dynasty - NEO",
            ),
            (
                "vow",
                VOW_SYMBOL,
                Length::Units(32),
                "Innistrad: Crimson Vow - VOW",
            ),
            (
                "mid",
                MID_SYMBOL,
                Length::Units(32),
                "Innistrad: Midnight Hunt - MID",
            ),
            (
                "afr",
                AFR_SYMBOL,
                Length::Units(32),
                "Adventures in the Forgotten Realms - AFR",
            ),
            (
                "stx",
                STX_SYMBOL,
                Length::Units(32),
                "Strixhaven: School of Mages - STX",
            ),
            ("khm", KHM_SYMBOL, Length::Units(32), "Kaldheim - KHM"),
            (
                "znr",
                ZNR_SYMBOL,
                Length::Units(32),
                "Zendikar Rising - ZNR",
            ),
            ("m21", M21_SYMBOL, Length::Units(24), "Core Set 2021 - M21"),
            (
                "iko",
                IKO_SYMBOL,
                Length::Units(32),
                "Ikoria: Lair of Behemoths - IKO",
            ),
            (
                "thb",
                THB_SYMBOL,
                Length::Units(32),
                "Theros Beyond Death - THB",
            ),
            (
                "eld",
                ELD_SYMBOL,
                Length::Units(32),
                "Throne of Eldraine - ELD",
            ),
            ("m20", M20_SYMBOL, Length::Units(24), "Core Set 2020 - M20"),
            (
                "war",
                WAR_SYMBOL,
                Length::Units(32),
                "War of the Spark - WAR",
            ),
            (
                "rna",
                RNA_SYMBOL,
                Length::Units(28),
                "Ravnica Allegiance - RNA",
            ),
            (
                "grn",
                GRN_SYMBOL,
                Length::Units(28),
                "Guilds of Ravnica - GRN",
            ),
            ("m19", M19_SYMBOL, Length::Units(24), "Core Set 2019 - M19"),
            ("dom", DOM_SYMBOL, Length::Units(32), "Dominaria - DOM"),
            (
                "rix",
                RIX_SYMBOL,
                Length::Units(32),
                "Rivals of Ixalan - RIX",
            ),
            ("xln", XLN_SYMBOL, Length::Units(32), "Ixalan - XLN"),
            (
                "klr",
                KLR_SYMBOL,
                Length::Units(32),
                "Kaladesh Remastered - KLR",
            ),
            (
                "akr",
                AKR_SYMBOL,
                Length::Units(32),
                "Amonkhet Remastered - AKR",
            ),
        ];
        let mut buttons_row = row();
        for (set, symbol, height, tooltip_text) in sets {
            let i = image(iced::pure::widget::image::Handle::from_memory(
                symbol.to_vec(),
            ))
            .height(height);
            let b = button(container(i))
                .on_press(TrackerMessage::SetSelector(
                    SetSelectorMessage::Interaction(Interaction::SetSelected(String::from(set))),
                ))
                .height(height);

            let t = tooltip(b, tooltip_text, iced::tooltip::Position::FollowCursor)
                .gap(5)
                .style(style::TooltipStyle);

            buttons_row = buttons_row.push(t);
        }

        buttons_row = buttons_row
            .spacing(8)
            .padding(5)
            .align_items(Alignment::Center);

        container(buttons_row).width(Length::Fill).into()
    }

    pub fn update(&mut self, message: SetSelectorMessage) -> Command<TrackerMessage> {
        match message {
            SetSelectorMessage::Interaction(Interaction::SetSelected(set)) => {
                self.set_selected = set;
                return Command::perform(
                    {
                        let set = self.set_selected.clone();
                        async move { set }
                    },
                    |set| TrackerMessage::Action(Action::ChangeSet(set)),
                );
            }
        }
    }
}
