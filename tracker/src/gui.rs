mod style;
mod update;

use std::env;
use std::time::{Duration, Instant};

use async_std::path::PathBuf;
use iced::alignment::{Horizontal, Vertical};
use iced::image::Handle;
use iced::pure::widget::Column;
use iced::pure::{button, column, container, image, row, scrollable, text, tooltip, Element};
use iced::{executor, pure::Application, Command, Settings};
use iced::{Alignment, Length};

use crate::assets::*;
use crate::configuration::ParseParams;
use crate::logwatcher::{LineParseResult, LogWatcher};
use crate::mtgadb::{MtgaDb, UserSession};
use crate::Result;

#[derive(Default)]
pub struct TrackerGuiConfig {
    collector_dll_path: PathBuf,
    database_path: PathBuf,
}

pub enum Status {
    Ok(String),
    Error(String),
}

pub struct TrackerGui {
    display_user_session: Option<UserSession>,
    log_user_session: Option<UserSession>,
    collector_path: PathBuf,
    status: Status,
    selected_set: String,
    messages: Vec<String>,
    log_watcher: Option<LogWatcher>,
    database: MtgaDb,
    highlighted_cards: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum TrackerInteraction {
    InjectPressed,
    SetSelected(String),
}

#[derive(Debug)]
pub enum TrackerMessage {
    TrackerInjected(Result<()>),
    UserLoggedIn(Result<UserSession>),
    TrackerInteraction(TrackerInteraction),
    ParseTriggered(Instant),
    Ok(String),
    Error((String, String)),
    ParseCompleted(Result<(LogWatcher, Vec<LineParseResult>)>),
    None(()),
    LogMessage(String),
}

impl Application for TrackerGui {
    type Executor = executor::Default;
    type Message = TrackerMessage;
    type Flags = TrackerGuiConfig;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let log_path: PathBuf = if cfg!(windows) {
            [
                env::var("APPDATA").unwrap().as_str(),
                "..",
                "LocalLow",
                "Wizards Of The Coast",
                "MTGA",
                "Player.log",
            ]
            .iter()
            .collect()
        } else {
            todo!()
        };

        let gui = TrackerGui {
            display_user_session: None,
            log_user_session: None,
            collector_path: flags.collector_dll_path.clone(),
            selected_set: String::from("DMU"),
            status: Status::Ok(String::from("Initializing...")),
            messages: vec![],
            log_watcher: Some(LogWatcher::new(log_path)),
            database: MtgaDb::new(flags.database_path.clone()),
            highlighted_cards: vec![],
        };

        let init_commands = vec![
            Command::perform(
                update::inject_tracker(flags.collector_dll_path.clone()),
                TrackerMessage::TrackerInjected,
            ),
            Command::perform(
                {
                    let database = flags.database_path.clone();
                    async move { update::get_user_session(database).await }
                },
                TrackerMessage::UserLoggedIn,
            ),
        ];

        (gui, Command::batch(init_commands))
    }

    fn title(&self) -> String {
        String::from("MTGA Tracker")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match update::handle_message(self, message) {
            Ok(message) => message,
            Err(e) => Command::perform(
                async move { (String::from("update"), e.to_string()) },
                TrackerMessage::Error,
            ),
        }
    }

    fn view(&self) -> Element<Self::Message> {
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

        let mut buttons = vec![];
        for (set, symbol, height, tooltip_text) in sets {
            let i = image(Handle::from_memory(symbol.to_vec())).height(height);
            let b: Element<TrackerInteraction> = button(i)
                .on_press(TrackerInteraction::SetSelected(String::from(set)))
                .height(height)
                .into();
            let t: Element<TrackerInteraction> =
                tooltip(b, tooltip_text, iced::tooltip::Position::FollowCursor)
                    .gap(5)
                    .style(style::TooltipStyle)
                    .into();

            buttons.push(t.map(TrackerMessage::TrackerInteraction));
        }

        let buttons_row = iced::pure::widget::Row::with_children(buttons)
            .spacing(8)
            .padding(5)
            .align_items(Alignment::Center);

        let messages: Element<TrackerInteraction> = Column::with_children(
            self.messages
                .iter()
                .map(|msg| text(msg).size(12).into())
                .collect::<Vec<Element<_>>>(),
        )
        .width(Length::Fill)
        .into();
        let scrollable_messages: Element<TrackerInteraction> = scrollable(messages).into();

        let cards_list: Element<TrackerInteraction> = Column::with_children(
            self.highlighted_cards
                .iter()
                .map(|msg| text(msg).size(12).into())
                .collect::<Vec<Element<_>>>(),
        )
        .width(Length::Fill)
        .into();
        let scrollable_cards: Element<TrackerInteraction> = scrollable(cards_list).into();

        let main_content: Element<TrackerInteraction> = row()
            .push(scrollable_messages)
            .push(scrollable_cards)
            .into();

        let main_content = column()
            .push(buttons_row)
            .push(main_content.map(TrackerMessage::TrackerInteraction))
            .height(Length::Fill);

        let status_message = {
            let inject_status_text = match &self.status {
                Status::Ok(value) => text(value)
                    .color([0.0, 0.0, 0.0])
                    .vertical_alignment(Vertical::Center),
                Status::Error(e) => text(e)
                    .color([1.0, 0.0, 0.0])
                    .vertical_alignment(Vertical::Center),
            };

            let selected_set_text = text(format!("Selected set: {}", self.selected_set))
                .vertical_alignment(Vertical::Center);

            let status_row = row()
                .push(inject_status_text)
                .push(selected_set_text)
                .align_items(Alignment::Center);
            status_row
        };

        let inject_button: Element<TrackerInteraction> = button("Inject")
            .on_press(TrackerInteraction::InjectPressed)
            .into();

        let status_bar = row()
            .push(inject_button.map(TrackerMessage::TrackerInteraction))
            .push(status_message)
            .height(Length::Shrink);

        let content = column().push(main_content).push(status_bar);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        iced::time::every(Duration::from_secs(1)).map(TrackerMessage::ParseTriggered)
    }
}

impl TrackerGui {
    pub fn set_status(&mut self, status: Status) {
        self.status = status;
    }
}

pub fn run(config: &ParseParams) -> Result<()> {
    let settings = Settings {
        flags: TrackerGuiConfig {
            collector_dll_path: config.collector_dll_path.clone(),
            database_path: config.database_path.clone(),
        },
        default_font: Some(OPEN_SANS_BOLD_FONT),
        default_text_size: 14,
        ..Default::default()
    };

    TrackerGui::run(settings)?;
    Ok(())
}
