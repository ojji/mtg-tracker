use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::gui::{Status, TrackerGui, TrackerInteraction, TrackerMessage};
use crate::mtgadb::{MtgaDb, UserSession};
use crate::{logwatcher, Result};

use async_std::path::PathBuf;
use iced::Command;
use injector::Injector;

pub fn handle_message(
    app: &mut TrackerGui,
    message: TrackerMessage,
) -> Result<Command<TrackerMessage>> {
    match message {
        TrackerMessage::TrackerInjected(r) => match r {
            Ok(_) => {
                return Ok(Command::perform(
                    async { String::from("Data collector injected successfully.") },
                    TrackerMessage::Ok,
                ));
            }
            Err(e) => {
                return Ok(Command::perform(
                    async move { (String::from("tracker-injected"), e.to_string()) },
                    TrackerMessage::Error,
                ));
            }
        },
        TrackerMessage::ParseTriggered(_) => {
            let log_watcher = app.log_watcher.take();
            if log_watcher.is_some() {
                return Ok(Command::perform(
                    async move { logwatcher::watch_log(log_watcher.unwrap()).await },
                    TrackerMessage::ParseCompleted,
                ));
            }
        }
        TrackerMessage::TrackerInteraction(TrackerInteraction::InjectPressed) => {
            return Ok(Command::perform(
                inject_tracker(app.collector_path.clone()),
                TrackerMessage::TrackerInjected,
            ));
        }
        TrackerMessage::TrackerInteraction(TrackerInteraction::SetSelected(s)) => {
            app.selected_set = s;
            let common_cards = app
                .database
                .get_common_cards_in_boosters(&app.selected_set)?;
            let uncommon_cards = app
                .database
                .get_uncommon_cards_in_boosters(&app.selected_set)?;
            let mythic_cards = app
                .database
                .get_mythic_cards_in_boosters(&app.selected_set)?;
            let rare_cards = app.database.get_rare_cards_in_boosters(&app.selected_set)?;
            let log_message = format!(
                "There are {} ({}) common, {} ({}) uncommon, {} ({}) rare, {} ({}) mythic cards in {}.",
                common_cards.len(),
                common_cards.len() * 4,
                uncommon_cards.len(),
                uncommon_cards.len() * 4,
                rare_cards.len(),
                rare_cards.len() * 4,
                mythic_cards.len(),
                mythic_cards.len() * 4,
                &app.selected_set
            );

            let user_id = match &app.display_user_session {
                Some(user) => user.user_id(),
                None => return Err("User id is not set".into()),
            };

            let mut collected_common_cards = app.database.get_collected_cards_in_boosters(
                user_id,
                &app.selected_set,
                "common",
            )?;
            collected_common_cards.sort_by(|a, b| a.0.name().partial_cmp(b.0.name()).unwrap());

            let mut collected_uncommon_cards = app.database.get_collected_cards_in_boosters(
                user_id,
                &app.selected_set,
                "uncommon",
            )?;
            collected_uncommon_cards.sort_by(|a, b| a.0.name().partial_cmp(b.0.name()).unwrap());

            let mut collected_rare_cards =
                app.database
                    .get_collected_cards_in_boosters(user_id, &app.selected_set, "rare")?;
            collected_rare_cards.sort_by(|a, b| a.0.name().partial_cmp(b.0.name()).unwrap());

            let mut collected_mythic_cards = app.database.get_collected_cards_in_boosters(
                user_id,
                &app.selected_set,
                "mythic",
            )?;
            collected_mythic_cards.sort_by(|a, b| a.0.name().partial_cmp(b.0.name()).unwrap());

            let cards = collected_common_cards
                .iter()
                .chain(collected_uncommon_cards.iter())
                .chain(collected_rare_cards.iter())
                .chain(collected_mythic_cards.iter())
                .map(|c| format!("{}/4 - {}", c.1, c.0.to_string()))
                .collect();

            app.highlighted_cards = cards;

            return Ok(Command::perform(
                async move { log_message },
                TrackerMessage::LogMessage,
            ));
        }
        TrackerMessage::Ok(value) => {
            app.set_status(Status::Ok(value));
        }
        TrackerMessage::Error((sender, error)) => {
            app.set_status(Status::Error(format!("Error from {}: {}.", sender, error)));
        }
        TrackerMessage::ParseCompleted(result) => {
            match result {
                Ok((watcher, parsed_lines)) => {
                    app.log_watcher = Some(watcher);

                    let commands = parsed_lines
                    .into_iter()
                    .map(|parsed_line| match parsed_line {
                        logwatcher::LineParseResult::None => {
                            Command::perform(async {}, TrackerMessage::None)
                        }
                        logwatcher::LineParseResult::UserSession(event) => {
                            let user_session =
                                match app.database.get_user_session(Some(event.attachment)) {
                                    Ok(user_session) => user_session,
                                    Err(e) => {
                                        return Command::perform(
                                            async move { (String::from("parser-usersession"), e.to_string()) },
                                            TrackerMessage::Error,
                                        )
                                    }
                                };

                            let message = format!("Current user: {}", user_session.screen_name());
                            app.log_user_session = Some(user_session);

                            Command::perform(async move { message }, TrackerMessage::LogMessage)
                        }
                        logwatcher::LineParseResult::CollectionEvent(event) => {
                            let current_user = match app.log_user_session.as_ref() {
                                Some(user) => user,
                                None => {
                                    return Command::perform(
                                        async move { (String::from("parser-collectionevent"), String::from("No user session is set")) },
                                        TrackerMessage::Error,
                                    )
                                }
                            };

                            let mut hasher = DefaultHasher::new();
                            event.attachment.hash(&mut hasher);
                            let collection_hash = hasher.finish() as i64;

                            if let Err(e) = app.database.add_user_collection_event(
                                current_user,
                                collection_hash,
                                event.timestamp.clone(),
                                event.attachment,
                            ) {
                                return Command::perform(
                                    async move { (String::from("parser-collectionevent"), e.to_string()) },
                                    TrackerMessage::Error,
                                );
                            };
                            return Command::perform(
                                {
                                    let timestamp = event.timestamp.clone();
                                    async move { format!("Collection updated at {}", timestamp) }
                                },
                                TrackerMessage::LogMessage,
                            );
                        }
                        logwatcher::LineParseResult::InventoryEvent(event) => {
                            let current_user = match app.log_user_session.as_ref() {
                                Some(user) => user,
                                None => {
                                    return Command::perform(
                                        async { (String::from("parser-inventoryupdateevent"), String::from("No user session is set")) },
                                        TrackerMessage::Error,
                                    )
                                }
                            };

                            let mut hasher = DefaultHasher::new();
                            event.hash(&mut hasher);
                            let inventory_hash = hasher.finish() as i64;

                            if let Err(e) = app.database.add_user_inventory_event(
                                current_user,
                                inventory_hash,
                                event.timestamp.clone(),
                                event.attachment,
                            ) {
                                return Command::perform(
                                    async move { (String::from("parser-inventoryevent"), e.to_string()) },
                                    TrackerMessage::Error,
                                );
                            };

                            return Command::perform(
                                {
                                    let timestamp = event.timestamp.clone();
                                    async move { format!("Player inventory event received at {}", timestamp) }
                                },
                                TrackerMessage::LogMessage,
                            );
                        },
                        logwatcher::LineParseResult::InventoryUpdateEvent(event) => {
                            let current_user = match app.log_user_session.as_ref() {
                                Some(user) => user,
                                None => {
                                    return Command::perform(
                                        async { (String::from("parser-inventoryupdateevent"), String::from("No user session is set")) },
                                        TrackerMessage::Error,
                                    )
                                }
                            };

                            let mut hasher = DefaultHasher::new();
                            event.hash(&mut hasher);
                            let inventory_update_hash = hasher.finish() as i64;

                            if let Err(e) = app.database.add_user_inventory_update_event(
                                current_user,
                                inventory_update_hash,
                                event.timestamp.clone(),
                                event.attachment,
                            ) {
                                return Command::perform(
                                    async move { (String::from("parser-inventoryupdateevent"), e.to_string()) },
                                    TrackerMessage::Error,
                                );
                            };

                            return Command::perform(
                                {
                                    let timestamp = event.timestamp.clone();
                                    async move { format!("Player inventory update event received at {}", timestamp) }
                                },
                                TrackerMessage::LogMessage,
                            );
                        }
                    })
                    .collect::<Vec<Command<TrackerMessage>>>();

                    return Ok(Command::batch(commands));
                }
                Err(e) => {
                    return Ok(Command::perform(
                        async move { (String::from("parser"), e.to_string()) },
                        TrackerMessage::Error,
                    ))
                }
            }
        }
        TrackerMessage::None(_) => {}
        TrackerMessage::LogMessage(message) => {
            app.messages.insert(0, message);
        }
        TrackerMessage::UserLoggedIn(user) => match user {
            Ok(user) => app.display_user_session = Some(user),
            Err(e) => {
                return Ok(Command::perform(
                    async move { (String::from("logged-in-user"), e.to_string()) },
                    TrackerMessage::Error,
                ))
            }
        },
    }
    Ok(Command::none())
}

pub async fn inject_tracker(collector_dll_path: PathBuf) -> Result<()> {
    let injector = Injector::new().await?;
    injector.inject_tracker(collector_dll_path).await?;
    Ok(())
}

pub async fn get_user_session(database_path: PathBuf) -> Result<UserSession> {
    let db = MtgaDb::new(database_path);
    db.get_user_session(None)
}
