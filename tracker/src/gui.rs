mod components;
mod style;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use std::env;

use async_std::path::PathBuf;
use iced::widget::{column, container, row};
use iced::{executor, Application, Command, Element, Font, Length, Settings, Theme};
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
    logview::LogViewComponent,
    setselector::{SetSelectorComponent, SetSelectorMessage},
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
    ParseLog,
    LogMessage(String, Severity),
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
    type Theme = Theme;

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
            logview_component: LogViewComponent::new(),
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
                |user| TrackerMessage::DisplayUserChanged(user),
            ),
        ];

        (gui, Command::batch(init_commands))
    }

    fn title(&self) -> String {
        String::from("MTGA Tracker")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            TrackerMessage::TrackerInjected(r) => match r {
                Ok(_) => {
                    self.inject_bar_component
                        .set_status(Status::Ok(String::from(
                            "Data collector injected successfully.",
                        )));
                    return Command::perform(async {}, |_| {
                        TrackerMessage::Action(Action::LogMessage(
                            String::from("Data collector injected successfully."),
                            Severity::Debug,
                        ))
                    });
                }
                Err(e) => {
                    self.inject_bar_component
                        .set_status(Status::Error(e.to_string()));
                    return Command::perform(async {}, move |_| {
                        TrackerMessage::Action(Action::LogMessage(
                            format!("Error injecting: {}", e.to_string()),
                            Severity::Error,
                        ))
                    });
                }
            },
            TrackerMessage::DisplayUserChanged(r) => match r {
                Ok(user) => {
                    self.display_user_session = Some(user.clone());
                    let mut commands = vec![];
                    match self.draft_summary_component.set_current_user(user.clone()) {
                        Ok(command) => commands.push(command),
                        Err(e) => commands.push(Command::perform(async {}, move |_| {
                            TrackerMessage::Action(Action::LogMessage(
                                e.to_string(),
                                Severity::Error,
                            ))
                        })),
                    }

                    match self.collection_component.set_current_user(user.clone()) {
                        Ok(command) => commands.push(command),
                        Err(e) => commands.push(Command::perform(async {}, move |_| {
                            TrackerMessage::Action(Action::LogMessage(
                                e.to_string(),
                                Severity::Error,
                            ))
                        })),
                    }

                    return Command::batch(commands);
                }
                Err(e) => {
                    return Command::perform(async {}, move |_| {
                        TrackerMessage::Action(Action::LogMessage(e.to_string(), Severity::Error))
                    })
                }
            },
            TrackerMessage::Action(action) => match action {
                Action::ParseLog => {
                    let log_watcher = self.log_watcher.take();
                    if log_watcher.is_some() {
                        return Command::perform(
                            async move { parser::watch_log(log_watcher.unwrap()).await },
                            TrackerMessage::ParseCompleted,
                        );
                    }
                }
                Action::LogMessage(message, severity) => {
                    self.logview_component.log_message(message.clone());
                    if let Some(e) = self.logger.as_mut() {
                        e.log(message.clone(), severity);
                    }

                    return Command::perform(async {}, TrackerMessage::None);
                }
                Action::ChangeSet(set) => {
                    let mut commands = vec![];
                    match self
                        .draft_summary_component
                        .change_selected_set(set.clone())
                    {
                        Ok(command) => commands.push(command),
                        Err(e) => {
                            commands.push(Command::perform(async {}, move |_| {
                                TrackerMessage::Action(Action::LogMessage(
                                    e.to_string(),
                                    Severity::Error,
                                ))
                            }));
                        }
                    }

                    match self.collection_component.change_selected_set(set.clone()) {
                        Ok(command) => commands.push(command),
                        Err(e) => {
                            commands.push(Command::perform(async {}, move |_| {
                                TrackerMessage::Action(Action::LogMessage(
                                    e.to_string(),
                                    Severity::Error,
                                ))
                            }));
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

                    let commands = parsed_messages.into_iter().map(|parsed_message| {
                        match parsed_message {
                            ParseResults::AccountInfoResult(r) => {
                                let user_session = match self.database.get_user_session(Some(r.user_id()), Some(r.screen_name())) {
                                    Ok(user_session) => user_session,
                                    Err(e) => {
                                        return Command::perform(
                                            async {},
                                            move |_| TrackerMessage::Action(Action::LogMessage(format!("parser-usersession error: {}", e.to_string()), Severity::Error))
                                        )
                                    }
                                };

                                let message = format!("Current user: {}", user_session.screen_name());
                                self.log_user_session = Some(user_session);

                                Command::perform(async {}, move |_| TrackerMessage::Action(Action::LogMessage(message, Severity::Info)))
                            },
                            ParseResults::InventoryUpdateResult(inventory_update) => {
                                let current_user = match self.log_user_session.as_ref() {
                                    Some(user) => user,
                                    None => {
                                        return Command::perform(async {}, move |_| TrackerMessage::Action(Action::LogMessage(String::from("parser-inventoryupdateevent error: No user session is set"), Severity::Error))); }
                                };

                                let mut hasher= DefaultHasher::new();
                                inventory_update.hash(&mut hasher);
                                let inventory_update_hash = hasher.finish() as i64;
                                let timestamp = inventory_update.get_date().unwrap();

                                if let Err(e) = self.database.add_user_inventory_update_event(current_user, inventory_update_hash, timestamp.to_string(), inventory_update.payload()) {
                                    return Command::perform(async {}, move |_| TrackerMessage::Action(Action::LogMessage(format!("parser-inventoryupdateevent error: {}", e.to_string()), Severity::Error)));
                                }

                                return Command::perform(async {}, move |_| TrackerMessage::Action(Action::LogMessage(format!("Player inventory update event received at {}", timestamp), Severity::Info)));
                            }
                            ParseResults::InventoryResult(inventory) => {
                                let current_user = match self.log_user_session.as_ref() {
                                    Some(user) => user,
                                    None => {
                                        return Command::perform(async {}, move |_| TrackerMessage::Action(Action::LogMessage(String::from("parser-inventoryevent error: No user session is set"), Severity::Error)));
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
                                    return Command::perform(
                                        async {},
                                        move |_| TrackerMessage::Action(Action::LogMessage(format!("parser-inventoryevent error: {}", e.to_string()), Severity::Error))
                                    );
                                };

                                return Command::perform(async {}, move |_| TrackerMessage::Action(Action::LogMessage(format!("Player inventory event received at {}", timestamp), Severity::Info)));
                            }
                            ParseResults::CollectionResult(collection) => {
                                let current_user = match self.log_user_session.as_ref() {
                                    Some(user) => user,
                                    None => {
                                        return Command::perform(async {}, |_| TrackerMessage::Action(Action::LogMessage(String::from("parser-collectionevent error: No user session is set"), Severity::Error)))
                                    },
                                };

                                let mut hasher = DefaultHasher::new();
                                collection.payload().hash(&mut hasher);
                                let collection_hash = hasher.finish() as i64;
                                let timestamp = collection.get_date().unwrap();

                                if let Err(e) = self.database.add_user_collection_event(current_user, collection_hash, timestamp.to_string(), collection.payload()) {
                                    return Command::perform(async {}, move |_| TrackerMessage::Action(Action::LogMessage(format!("parser-collectionevent error: {}", e.to_string()), Severity::Error)));
                                }

                                return Command::perform(async {}, move |_| TrackerMessage::Action(Action::LogMessage(format!("Collection updated at {}", timestamp.to_string()), Severity::Info)));
                            }
                            ParseResults::SceneChangeResult(scene_change) => {
                                return Command::perform(async {}, move |_| TrackerMessage::Action(Action::LogMessage(format!("Scene change from {} to {} ctx: {}", scene_change.from_scene(), scene_change.to_scene(), scene_change.context().unwrap()), Severity::Info)));
                            }
                            _ => {
                                Command::none()
                            }
                            // ParseResults::UnknownResult(_) => todo!(),
                        }
                    }).collect::<Vec<Command<TrackerMessage>>>();
                    return Command::batch(commands);
                }
                Err(e) => {
                    return Command::perform(async {}, move |_| {
                        TrackerMessage::Action(Action::LogMessage(e.to_string(), Severity::Error))
                    })
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
                LoggerEvent::Error(e) => {
                    TrackerMessage::Action(Action::LogMessage(e, Severity::Error))
                }
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
        default_font: Font::DEFAULT,
        default_text_size: 14.0,
        ..Default::default()
    };

    TrackerGui::run(settings)?;
    Ok(())
}
