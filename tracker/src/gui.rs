mod components;
mod style;

use std::collections::hash_map::DefaultHasher;
use std::env;
use std::hash::{Hash, Hasher};
use std::time::Duration;

use async_std::path::PathBuf;
use iced::{
    executor,
    pure::{column, container, row, Application, Element},
    Command, Length, Settings,
};
use injector::Injector;

use crate::{
    assets::*,
    configuration::ParseParams,
    logwatcher,
    logwatcher::{LineParseResult, LogWatcher},
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
}

#[derive(Debug, Clone)]
pub enum TrackerMessage {
    TrackerInjected(Result<()>),
    DisplayUserChanged(Result<UserSession>),
    ParseCompleted(Result<(LogWatcher, Vec<LineParseResult>)>),
    Action(Action),
    SetSelector(SetSelectorMessage),
}

#[derive(Debug, Clone)]
pub enum Action {
    Reinject,
    ChangeSet(String),
    ParseLog,
    LogMessage(String),
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
                    println!("Inject success");
                    self.inject_bar_component
                        .set_status(Status::Ok(String::from(
                            "Data collector injected successfully.",
                        )));
                }
                Err(e) => {
                    println!("Error injecting");
                    self.inject_bar_component
                        .set_status(Status::Error(e.to_string()));
                }
            },
            TrackerMessage::DisplayUserChanged(r) => match r {
                Ok(user) => {
                    self.display_user_session = Some(user.clone());
                    let mut commands = vec![];
                    match self.draft_summary_component.set_current_user(user.clone()) {
                        Ok(command) => commands.push(command),
                        Err(e) => commands
                            .push(Command::perform(async move { e.to_string() }, |error| {
                                TrackerMessage::Action(Action::LogMessage(error))
                            })),
                    }

                    match self.collection_component.set_current_user(user.clone()) {
                        Ok(command) => commands.push(command),
                        Err(e) => commands
                            .push(Command::perform(async move { e.to_string() }, |error| {
                                TrackerMessage::Action(Action::LogMessage(error))
                            })),
                    }

                    return Command::batch(commands);
                }
                Err(e) => {
                    return Command::perform(async move { e.to_string() }, |error| {
                        TrackerMessage::Action(Action::LogMessage(error))
                    })
                }
            },
            TrackerMessage::Action(action) => match action {
                Action::ParseLog => {
                    let log_watcher = self.log_watcher.take();
                    if log_watcher.is_some() {
                        return Command::perform(
                            async move { logwatcher::watch_log(log_watcher.unwrap()).await },
                            TrackerMessage::ParseCompleted,
                        );
                    }
                }
                Action::LogMessage(message) => {
                    println!("Log message: {}", message);
                    self.logview_component.log_message(message);
                }
                Action::ChangeSet(set) => {
                    let mut commands = vec![];
                    match self
                        .draft_summary_component
                        .change_selected_set(set.clone())
                    {
                        Ok(command) => commands.push(command),
                        Err(e) => {
                            commands
                                .push(Command::perform(async move { e.to_string() }, |error| {
                                    TrackerMessage::Action(Action::LogMessage(error))
                                }));
                        }
                    }

                    match self.collection_component.change_selected_set(set.clone()) {
                        Ok(command) => commands.push(command),
                        Err(e) => {
                            commands
                                .push(Command::perform(async move { e.to_string() }, |error| {
                                    TrackerMessage::Action(Action::LogMessage(error))
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
                Ok((watcher, parsed_lines)) => {
                    self.log_watcher = Some(watcher);

                    let commands = parsed_lines
                    .into_iter()
                    .map(|parsed_line| match parsed_line {
                        logwatcher::LineParseResult::None => {
                            Command::none()
                        }
                        logwatcher::LineParseResult::UserSession(event) => {
                            let user_session =
                                match self.database.get_user_session(Some(event.attachment)) {
                                    Ok(user_session) => user_session,
                                    Err(e) => {
                                        return Command::perform(
                                            async move { format!("parser-usersession error: {}", e.to_string())},
                                            |error| TrackerMessage::Action(Action::LogMessage(error))
                                        )
                                    }
                                };

                            let message = format!("Current user: {}", user_session.screen_name());
                            self.log_user_session = Some(user_session);

                            Command::perform(async move { message }, |message| TrackerMessage::Action(Action::LogMessage(message)))
                        }
                        logwatcher::LineParseResult::CollectionEvent(event) => {
                            let current_user = match self.log_user_session.as_ref() {
                                Some(user) => user,
                                None => {
                                    return Command::perform(
                                        async move { String::from("parser-collectionevent error: No user session is set") },
                                        |error| TrackerMessage::Action(Action::LogMessage(error))
                                    )
                                }
                            };

                            let mut hasher = DefaultHasher::new();
                            event.attachment.hash(&mut hasher);
                            let collection_hash = hasher.finish() as i64;

                            if let Err(e) = self.database.add_user_collection_event(
                                current_user,
                                collection_hash,
                                event.timestamp.clone(),
                                event.attachment,
                            ) {
                                return Command::perform(
                                    async move { format!("parser-collectionevent error: {}", e.to_string()) },
                                    |error| TrackerMessage::Action(Action::LogMessage(error))
                                )
                            };

                            return Command::perform(
                                {
                                    let timestamp = event.timestamp.clone();
                                    async move { format!("Collection updated at {}", timestamp) }
                                },
                                |message| TrackerMessage::Action(Action::LogMessage(message))
                            );
                        }
                        logwatcher::LineParseResult::InventoryEvent(event) => {
                            let current_user = match self.log_user_session.as_ref() {
                                Some(user) => user,
                                None => {
                                    return Command::perform(
                                        async move { String::from("parser-inventoryevent error: No user session is set") },
                                        |error| TrackerMessage::Action(Action::LogMessage(error))
                                    )
                                }
                            };

                            let mut hasher = DefaultHasher::new();
                            event.hash(&mut hasher);
                            let inventory_hash = hasher.finish() as i64;

                            if let Err(e) = self.database.add_user_inventory_event(
                                current_user,
                                inventory_hash,
                                event.timestamp.clone(),
                                event.attachment,
                            ) {
                                return Command::perform(
                                    async move { format!("parser-inventoryevent error: {}", e.to_string()) },
                                    |error| TrackerMessage::Action(Action::LogMessage(error))
                                );
                            };

                            return Command::perform(
                                {
                                    let timestamp = event.timestamp.clone();
                                    async move { format!("Player inventory event received at {}", timestamp) }
                                },
                                |message| TrackerMessage::Action(Action::LogMessage(message)),
                            );
                        },
                        logwatcher::LineParseResult::InventoryUpdateEvent(event) => {
                            let current_user = match self.log_user_session.as_ref() {
                                Some(user) => user,
                                None => {
                                    return Command::perform(
                                        async move { String::from("parser-inventoryupdateevent error: No user session is set") },
                                        |error| TrackerMessage::Action(Action::LogMessage(error))
                                    )
                                }
                            };

                            let mut hasher = DefaultHasher::new();
                            event.hash(&mut hasher);
                            let inventory_update_hash = hasher.finish() as i64;

                            if let Err(e) = self.database.add_user_inventory_update_event(
                                current_user,
                                inventory_update_hash,
                                event.timestamp.clone(),
                                event.attachment,
                            ) {
                                return Command::perform(
                                    async move { format!("parser-inventoryupdateevent error: {}", e.to_string()) },
                                    |error| TrackerMessage::Action(Action::LogMessage(error)),
                                );
                            };

                            return Command::perform(
                                {
                                    let timestamp = event.timestamp.clone();
                                    async move { format!("Player inventory update event received at {}", timestamp) }
                                },
                                |message| TrackerMessage::Action(Action::LogMessage(message)),
                            );
                        }
                    })
                    .collect::<Vec<Command<TrackerMessage>>>();

                    return Command::batch(commands);
                }
                Err(e) => {
                    return Command::perform(async move { e.to_string() }, |error| {
                        TrackerMessage::Action(Action::LogMessage(error))
                    })
                }
            },
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

        let content = column()
            .push(set_selector)
            .push(
                container(
                    row()
                        .push(self.draft_summary_component.view())
                        .push(side_view),
                )
                .height(Length::Fill)
                .width(Length::Fill),
            )
            .push(status_bar);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        iced::time::every(Duration::from_secs(1)).map(|_| TrackerMessage::Action(Action::ParseLog))
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
        db.get_user_session(None)
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
        default_font: Some(OPEN_SANS_BOLD_FONT),
        default_text_size: 14,
        ..Default::default()
    };

    TrackerGui::run(settings)?;
    Ok(())
}