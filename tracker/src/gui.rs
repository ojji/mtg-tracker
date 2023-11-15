mod components;
mod style;

use std::collections::hash_map::DefaultHasher;
use std::env;
use std::hash::{Hash, Hasher};
use std::time::Duration;

use async_std::path::PathBuf;
use iced::widget::{column, container, row};
use iced::{executor, Application, Command, Font, Length, Settings};
use iced_futures::Subscription;
use injector::Injector;

use crate::logger::{LogService, Logger, LoggerEvent, Severity};
use crate::mtgadb::model::{ParseResult, ParseResults};
use crate::parser;
use crate::{
    configuration::ParseParams,
    logwatcher::LogWatcher,
    mtgadb::{MtgaDb, UserSession},
    Result,
};

use components::{
    collection::CollectionComponent,
    draftsummary::DraftSummaryComponent,
    injectbar::{InjectBarComponent, Status},
    logview::{LogEntry, LogViewComponent},
    setselector::{SetSelectorComponent, SetSelectorMessage},
    Element,
};

use style::TrackerTheme;

#[derive(Default)]
pub struct TrackerGuiConfig {
    collector_dll_path: PathBuf,
    database_path: PathBuf,
}

pub struct TrackerGui {
    draft_summary_component: DraftSummaryComponent,
    set_selector_component: SetSelectorComponent,
    inject_bar_component: InjectBarComponent,
    logview_component: LogViewComponent,
    collection_component: CollectionComponent,
    mode: Mode,
    log_user_session: Option<UserSession>,
    display_user_session: Option<UserSession>,
    collector_path: PathBuf,
    log_watcher: Option<LogWatcher>,
    database: MtgaDb,
    logger: Option<Logger>,
}

#[derive(Debug, Clone)]
pub enum TrackerMessage {
    TrackerInjected(Result<()>),
    DisplayUserChanged(Result<UserSession>),
    CardImageLoaded(Result<(u32, iced::widget::image::Handle)>),
    ParseCompleted(Result<(LogWatcher, Vec<ParseResults>)>),
    Action(Action),
    SetSelector(SetSelectorMessage),
    LoggerInit(Logger),
    None(()),
}

#[derive(Debug, Clone)]
pub enum Action {
    Reinject,
    ChangeSet(String),
    CardHovered(u32),
    ParseLog,
    SwitchView,
}

#[derive(Debug, Clone)]
pub enum Mode {
    ShowLog,
    ShowCollection,
}

impl Mode {
    pub fn next(&self) -> Mode {
        match self {
            Mode::ShowLog => Mode::ShowCollection,
            Mode::ShowCollection => Mode::ShowLog,
        }
    }
}

impl Application for TrackerGui {
    type Executor = executor::Default;
    type Message = TrackerMessage;
    type Flags = TrackerGuiConfig;
    type Theme = TrackerTheme;

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
            draft_summary_component: DraftSummaryComponent::new(flags.database_path.as_path()),
            set_selector_component: SetSelectorComponent::new(),
            inject_bar_component: InjectBarComponent::new(),
            logview_component: LogViewComponent::new(flags.database_path.as_path()),
            collection_component: CollectionComponent::new(flags.database_path.as_path()),
            mode: Mode::ShowLog,
            log_user_session: None,
            display_user_session: None,
            collector_path: flags.collector_dll_path.clone(),
            log_watcher: Some(LogWatcher::new(log_path)),
            database: MtgaDb::new(flags.database_path.clone()),
            logger: None,
        };

        let init_commands = vec![
            Command::perform(
                TrackerGui::inject_tracker(flags.collector_dll_path.clone()),
                TrackerMessage::TrackerInjected,
            ),
            Command::perform(
                {
                    let database = flags.database_path.clone();
                    async move { TrackerGui::get_user_session(database).await }
                },
                TrackerMessage::DisplayUserChanged,
            ),
        ];

        (gui, Command::batch(init_commands))
    }

    fn title(&self) -> String {
        String::from("MTGA Tracker")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            TrackerMessage::CardImageLoaded(r) => match r {
                Ok((id, image_handle)) => {
                    return self.collection_component.image_loaded(id, image_handle);
                }
                Err(e) => {
                    if let Some(logger) = self.logger.as_mut() {
                        logger.log(e.to_string(), Severity::Error);
                    }
                    return Command::none();
                }
            },
            TrackerMessage::TrackerInjected(r) => match r {
                Ok(_) => {
                    let status_msg = "Data collector injected successfully.";
                    self.inject_bar_component
                        .set_status(Status::Ok(String::from(status_msg)));

                    if let Some(logger) = self.logger.as_mut() {
                        logger.log(String::from(status_msg), Severity::Debug);
                    }
                    self.logview_component
                        .add_entry(LogEntry::String(String::from(status_msg)));

                    return Command::none();
                }
                Err(e) => {
                    self.inject_bar_component
                        .set_status(Status::Error(e.to_string()));
                    if let Some(logger) = self.logger.as_mut() {
                        logger.log(format!("Error injecting: {}", e), Severity::Error);
                    }
                    return Command::none();
                }
            },
            TrackerMessage::DisplayUserChanged(r) => match r {
                Ok(user) => {
                    self.display_user_session = Some(user.clone());
                    let mut commands = vec![];
                    match self.draft_summary_component.set_current_user(user.clone()) {
                        Ok(command) => commands.push(command),
                        Err(e) => {
                            if let Some(logger) = self.logger.as_mut() {
                                logger.log(e.to_string(), Severity::Error);
                            }
                        }
                    }

                    match self.collection_component.set_current_user(user.clone()) {
                        Ok(command) => commands.push(command),
                        Err(e) => {
                            if let Some(logger) = self.logger.as_mut() {
                                logger.log(e.to_string(), Severity::Error);
                            }
                        }
                    }

                    return Command::batch(commands);
                }
                Err(e) => {
                    if let Some(logger) = self.logger.as_mut() {
                        logger.log(e.to_string(), Severity::Error);
                    }
                    return Command::none();
                }
            },
            TrackerMessage::Action(action) => match action {
                Action::ParseLog => {
                    let log_watcher = self.log_watcher.take();
                    if let Some(log_watcher) = log_watcher {
                        return Command::perform(
                            async move { parser::watch_log(log_watcher).await },
                            TrackerMessage::ParseCompleted,
                        );
                    }
                }
                Action::CardHovered(arena_id) => {
                    return self.collection_component.hover_card(arena_id);
                }
                Action::ChangeSet(set) => {
                    let mut commands = vec![];
                    match self
                        .draft_summary_component
                        .change_selected_set(set.clone())
                    {
                        Ok(command) => commands.push(command),
                        Err(e) => {
                            if let Some(logger) = self.logger.as_mut() {
                                logger.log(e.to_string(), Severity::Error);
                            }
                        }
                    }

                    match self.collection_component.change_selected_set(set.clone()) {
                        Ok(command) => commands.push(command),
                        Err(e) => {
                            if let Some(logger) = self.logger.as_mut() {
                                logger.log(e.to_string(), Severity::Error);
                            }
                        }
                    }

                    return Command::batch(commands);
                }
                Action::Reinject => {
                    return Command::perform(
                        TrackerGui::inject_tracker(self.collector_path.clone()),
                        TrackerMessage::TrackerInjected,
                    )
                }
                Action::SwitchView => self.mode = self.mode.next(),
            },
            TrackerMessage::SetSelector(message) => {
                return self.set_selector_component.update(message)
            }
            TrackerMessage::ParseCompleted(r) => match r {
                Ok((watcher, parsed_messages)) => {
                    self.log_watcher = Some(watcher);

                    parsed_messages.into_iter().for_each(|parsed_message| {
                        match parsed_message {
                            ParseResults::AccountInfoResult(r) => {
                                let user_session = match self.database.get_user_session(Some(r.user_id()), Some(r.screen_name())) {
                                    Ok(user_session) => user_session,
                                    Err(e) => {
                                        if let Some(logger) = self.logger.as_mut() {
                                            logger.log(format!("parser-usersession error: {}", e), Severity::Error);
                                        }
                                        return;
                                    }
                                };

                                let message = format!("Current user: {}", user_session.screen_name());
                                self.log_user_session = Some(user_session);

                                if let Some(logger) = self.logger.as_mut() {
                                    logger.log(message.clone(), Severity::Info);
                                }

                                self.logview_component.add_entry(LogEntry::String(message));
                            },
                            ParseResults::InventoryUpdateResult(inventory_update) => {
                                let current_user = match self.log_user_session.as_ref() {
                                    Some(user) => user,
                                    None => {
                                        if let Some(logger) = self.logger.as_mut() {
                                            logger.log(String::from("parser-inventoryupdateevent error: No user session is set"), Severity::Error);
                                        }
                                        return;
                                    }
                                };

                                let mut hasher= DefaultHasher::new();
                                inventory_update.hash(&mut hasher);
                                let inventory_update_hash = hasher.finish() as i64;
                                let timestamp = inventory_update.get_date().unwrap();

                                if let Err(e) = self.database.add_user_inventory_update_event(current_user, inventory_update_hash, timestamp.to_string(), inventory_update.payload()) {
                                    if let Some(logger) = self.logger.as_mut() {
                                        logger.log(format!("parser-inventoryupdateevent error: {}", e), Severity::Error);
                                    }
                                    return;
                                }

                                if let Some(logger) = self.logger.as_mut() {
                                    logger.log(format!("[{}] Player inventory update event received:\n`{}`", inventory_update.friendly_time(), inventory_update.get_content()), Severity::Info);
                                }
                                self.logview_component.add_entry(LogEntry::InventoryUpdate(inventory_update));
                            }
                            ParseResults::InventoryResult(inventory) => {
                                let current_user = match self.log_user_session.as_ref() {
                                    Some(user) => user,
                                    None => {
                                        if let Some(logger) = self.logger.as_mut() {
                                            logger.log("parser-inventoryevent error: No user session is set".to_string(), Severity::Error);
                                        }
                                        return;
                                    }
                                };

                                let mut hasher = DefaultHasher::new();
                                inventory.hash(&mut hasher);
                                let inventory_hash = hasher.finish() as i64;
                                let timestamp = inventory.get_date().unwrap();

                                if let Err(e) = self.database.add_user_inventory_event(
                                    current_user,
                                    inventory_hash,
                                    timestamp.to_string(),
                                    inventory.payload(),
                                ) {
                                    if let Some(logger) = self.logger.as_mut() {
                                        logger.log(format!("parser-inventoryevent error: {}", e), Severity::Error);
                                    }
                                    return;
                                };

                                let message = format!("[{}] Player inventory updated.", inventory.friendly_time());
                                if let Some(logger) = self.logger.as_mut() {
                                    logger.log(message.clone(), Severity::Info);
                                }

                                self.logview_component.add_entry(LogEntry::String(message));
                            }
                            ParseResults::CollectionResult(collection) => {
                                let current_user = match self.log_user_session.as_ref() {
                                    Some(user) => user,
                                    None => {
                                        if let Some(logger) = self.logger.as_mut() {
                                            logger.log("parser-collectionevent error: No user session is set".to_string(), Severity::Error);
                                        }
                                        return;
                                    },
                                };

                                let mut hasher = DefaultHasher::new();
                                collection.payload().hash(&mut hasher);
                                let collection_hash = hasher.finish() as i64;
                                let timestamp = collection.get_date().unwrap();

                                if let Err(e) = self.database.add_user_collection_event(current_user, collection_hash, timestamp.to_string(), collection.payload()) {
                                    if let Some(logger) = self.logger.as_mut() {
                                        logger.log(format!("parser-collectionevent error: {}", e), Severity::Error);
                                    }
                                    return;
                                }

                                let message = format!("[{}] Collection updated.", collection.friendly_time());
                                if let Some(logger) = self.logger.as_mut() {
                                    logger.log(message.clone(), Severity::Info);
                                }
                                self.logview_component.add_entry(LogEntry::String(message));
                            }
                            ParseResults::SceneChangeResult(scene_change) => {
                                if let Some(logger) = self.logger.as_mut() {
                                    logger.log(format!("Scene change from {} to {} ctx: {}", scene_change.from_scene(), scene_change.to_scene(), scene_change.context().unwrap()), Severity::Info);
                                }
                                self.logview_component.add_entry(LogEntry::String(format!("Scene change from {} to {} ctx: {}", scene_change.from_scene(), scene_change.to_scene(), scene_change.context().unwrap())));
                            }
                            _ => {}
                            // ParseResults::UnknownResult(_) => todo!(),
                        }
                    });
                }
                Err(e) => {
                    if let Some(logger) = self.logger.as_mut() {
                        logger.log(e.to_string(), Severity::Error);
                    }
                }
            },
            TrackerMessage::LoggerInit(sender) => {
                self.logger = Some(sender);
            }
            TrackerMessage::None(_) => {}
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let set_selector = self.set_selector_component.view();

        let status_bar = self.inject_bar_component.view();

        let side_view = match self.mode {
            Mode::ShowLog => self.logview_component.view(),
            Mode::ShowCollection => self.collection_component.view(),
        };

        let content = column![
            set_selector,
            container(row![self.draft_summary_component.view(), side_view],)
                .height(Length::Fill)
                .width(Length::Fill),
            status_bar
        ];

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        Subscription::batch(vec![
            iced::time::every(Duration::from_secs(1))
                .map(|_| TrackerMessage::Action(Action::ParseLog)),
            LogService::init().map(|e| match e {
                LoggerEvent::Initialized(logger) => TrackerMessage::LoggerInit(logger),
                LoggerEvent::Error(_) => TrackerMessage::None(()),
            }),
        ])
    }
}

impl TrackerGui {
    pub async fn inject_tracker(collector_dll_path: PathBuf) -> Result<()> {
        let injector = Injector::new().await?;
        injector.inject_tracker(collector_dll_path).await?;
        Ok(())
    }

    pub async fn get_user_session(database_path: PathBuf) -> Result<UserSession> {
        let db = MtgaDb::new(database_path);
        db.get_user_session(None, None)
    }
}

pub fn run(config: &ParseParams) -> Result<()> {
    let settings = Settings {
        flags: TrackerGuiConfig {
            collector_dll_path: config.collector_dll_path.clone(),
            database_path: config.database_path.clone(),
        },
        window: iced::window::Settings {
            min_size: Some((1150, 800)),
            //TODO: app icon icon: todo!(),
            ..Default::default()
        },
        default_font: Font {
            family: iced::font::Family::SansSerif,
            weight: iced::font::Weight::Bold,
            stretch: iced::font::Stretch::Normal,
            monospaced: false,
        },
        default_text_size: 11.0,
        ..Default::default()
    };

    TrackerGui::run(settings)?;
    Ok(())
}
