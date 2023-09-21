use iced::{
    widget::{button, container, image, row, tooltip, tooltip::Position},
    Alignment, Command, Element, Length,
};

use crate::{
    assets::*,
    gui::{Action, TrackerMessage},
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
            set_selected: String::from("ltr"),
        }
    }

    pub fn view(&self) -> Element<TrackerMessage> {
        let sets = vec![
            (
                "ltr",
                LTR_SYMBOL,
                Length::Fixed(28.0),
                "The Lord of the Rings: Tales of Middle-earth - LTR",
            ),
            (
                "mom",
                MOM_SYMBOL,
                Length::Fixed(32.0),
                "March of the Machine - MOM",
            ),
            (
                "one",
                ONE_SYMBOL,
                Length::Fixed(32.0),
                "Phyrexia: All Will Be One - ONE",
            ),
            (
                "bro",
                BRO_SYMBOL,
                Length::Fixed(32.0),
                "The Brothers' War - BRO",
            ),
            (
                "dmu",
                DMU_SYMBOL,
                Length::Fixed(32.0),
                "Dominaria United - DMU",
            ),
            (
                "hbg",
                HBG_SYMBOL,
                Length::Fixed(32.0),
                "Alchemy Horizons: Baldur's Gate - HBG",
            ),
            (
                "snc",
                SNC_SYMBOL,
                Length::Fixed(28.0),
                "Streets of New Capenna - SNC",
            ),
            (
                "neo",
                NEO_SYMBOL,
                Length::Fixed(28.0),
                "Kamigawa: Neon Dynasty - NEO",
            ),
            (
                "vow",
                VOW_SYMBOL,
                Length::Fixed(32.0),
                "Innistrad: Crimson Vow - VOW",
            ),
            (
                "mid",
                MID_SYMBOL,
                Length::Fixed(32.0),
                "Innistrad: Midnight Hunt - MID",
            ),
            (
                "afr",
                AFR_SYMBOL,
                Length::Fixed(32.0),
                "Adventures in the Forgotten Realms - AFR",
            ),
            (
                "stx",
                STX_SYMBOL,
                Length::Fixed(32.0),
                "Strixhaven: School of Mages - STX",
            ),
            ("khm", KHM_SYMBOL, Length::Fixed(32.0), "Kaldheim - KHM"),
            (
                "znr",
                ZNR_SYMBOL,
                Length::Fixed(32.0),
                "Zendikar Rising - ZNR",
            ),
            (
                "m21",
                M21_SYMBOL,
                Length::Fixed(24.0),
                "Core Set 2021 - M21",
            ),
            (
                "iko",
                IKO_SYMBOL,
                Length::Fixed(32.0),
                "Ikoria: Lair of Behemoths - IKO",
            ),
            (
                "thb",
                THB_SYMBOL,
                Length::Fixed(32.0),
                "Theros Beyond Death - THB",
            ),
            (
                "eld",
                ELD_SYMBOL,
                Length::Fixed(32.0),
                "Throne of Eldraine - ELD",
            ),
            (
                "m20",
                M20_SYMBOL,
                Length::Fixed(24.0),
                "Core Set 2020 - M20",
            ),
            (
                "war",
                WAR_SYMBOL,
                Length::Fixed(32.0),
                "War of the Spark - WAR",
            ),
            (
                "rna",
                RNA_SYMBOL,
                Length::Fixed(28.0),
                "Ravnica Allegiance - RNA",
            ),
            (
                "grn",
                GRN_SYMBOL,
                Length::Fixed(28.0),
                "Guilds of Ravnica - GRN",
            ),
            (
                "m19",
                M19_SYMBOL,
                Length::Fixed(24.0),
                "Core Set 2019 - M19",
            ),
            ("dom", DOM_SYMBOL, Length::Fixed(32.0), "Dominaria - DOM"),
            (
                "rix",
                RIX_SYMBOL,
                Length::Fixed(32.0),
                "Rivals of Ixalan - RIX",
            ),
            ("xln", XLN_SYMBOL, Length::Fixed(32.0), "Ixalan - XLN"),
            (
                "klr",
                KLR_SYMBOL,
                Length::Fixed(32.0),
                "Kaladesh Remastered - KLR",
            ),
            (
                "akr",
                AKR_SYMBOL,
                Length::Fixed(32.0),
                "Amonkhet Remastered - AKR",
            ),
        ];
        // let mut buttons_row = row();
        // for (set, symbol, height, tooltip_text) in sets {
        //     let i = image(iced::widget::image::Handle::from_memory(symbol.to_vec())).height(height);
        //     let b = button(container(i))
        //         .on_press(TrackerMessage::SetSelector(
        //             SetSelectorMessage::Interaction(Interaction::SetSelected(String::from(set))),
        //         ))
        //         .height(height);

        //     let t = tooltip(b, tooltip_text, Position::FollowCursor)
        //         .gap(5)
        //         .style(style::TooltipStyle);

        //     buttons_row = buttons_row.push(t);
        // }

        // buttons_row = buttons_row
        //     .spacing(8)
        //     .padding(5)
        //     .align_items(Alignment::Center);

        let buttons_row = row(sets
            .iter()
            .map(|(set, symbol, height, tooltip_text)| {
                let i =
                    image(iced::widget::image::Handle::from_memory(symbol.to_vec())).height(*height);
                let b = button(container(i))
                    .on_press(TrackerMessage::SetSelector(
                        SetSelectorMessage::Interaction(Interaction::SetSelected(String::from(
                            *set,
                        ))),
                    ))
                    .height(*height);
                let t = tooltip(b, tooltip_text, Position::FollowCursor)
                    .gap(5);
                    // .style(style::TooltipStyle);
                t.into()
            })
            .collect())
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
