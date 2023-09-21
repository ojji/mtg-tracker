use std::env;

use async_std::{fs::File, io::WriteExt};
use iced_futures::{
    futures::{
        channel::mpsc::{self, unbounded, UnboundedReceiver},
        SinkExt, StreamExt,
    },
    subscription, Subscription,
};

#[derive(Debug, Clone)]
pub enum Severity {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone)]
pub struct LogMessage {
    content: String,
    severity: Severity,
}

#[derive(Debug, Clone)]
pub struct LogService {}

enum LogServiceState {
    Starting(File, File, File),
    Initialized(File, File, File, UnboundedReceiver<LogMessage>),
    Error(String),
}

impl LogService {
    pub fn init() -> Subscription<LoggerEvent> {
        subscription::channel(
            std::any::TypeId::of::<LogService>(),
            100,
            |mut output| async move {
                let binding = env::current_exe().unwrap();
                let current_dir = binding.parent().unwrap();
                let debug_file_path = current_dir.join("debug.log");
                let info_file_path = current_dir.join("info.log");
                let error_file_path = current_dir.join("error.log");

                let mut state = 'init: {
                    let debug_file = File::create(debug_file_path).await;
                    if let Err(e) = debug_file {
                        break 'init LogServiceState::Error(format!(
                            "Could not create debug file: {}",
                            e.to_string()
                        ));
                    }

                    let info_file = File::create(info_file_path).await;
                    if let Err(e) = info_file {
                        break 'init LogServiceState::Error(format!(
                            "Could not create info file: {}",
                            e.to_string()
                        ));
                    }

                    let error_file = File::create(error_file_path).await;
                    if let Err(e) = error_file {
                        break 'init LogServiceState::Error(format!(
                            "Could not create error file: {}",
                            e.to_string()
                        ));
                    }

                    LogServiceState::Starting(
                        info_file.unwrap(),
                        debug_file.unwrap(),
                        error_file.unwrap(),
                    )
                };

                loop {
                    match &mut state {
                        LogServiceState::Starting(info_file, debug_file, error_file) => {
                            let (sender, receiver) = unbounded::<LogMessage>();
                            let _ = output.send(LoggerEvent::Initialized(Logger(sender))).await;
                            state = LogServiceState::Initialized(
                                info_file.to_owned(),
                                debug_file.to_owned(),
                                error_file.to_owned(),
                                receiver,
                            );
                        }
                        LogServiceState::Initialized(
                            info_file,
                            debug_file,
                            error_file,
                            receiver,
                        ) => {
                            let message = receiver.select_next_some().await;
                            match message.severity {
                                Severity::Debug => {
                                    let message = format!("[debug]{}\n", message.content);
                                    let _ = debug_file.write_all(message.as_bytes()).await;
                                    let _ = debug_file.flush().await;
                                }
                                Severity::Info => {
                                    let message = format!("[info]{}\n", message.content);
                                    let _ = info_file.write_all(message.as_bytes()).await;
                                    let _ = info_file.flush().await;
                                }
                                Severity::Warn => {
                                    let message = format!("[warn]{}\n", message.content);
                                    let _ = error_file.write_all(message.as_bytes()).await;
                                    let _ = error_file.flush().await;
                                }
                                Severity::Error => {
                                    let message = format!("[error]{}\n", message.content);
                                    let _ = error_file.write_all(message.as_bytes()).await;
                                    let _ = error_file.flush().await;
                                }
                            }
                        }
                        LogServiceState::Error(e) => {
                            let _ = output.send(LoggerEvent::Error(e.clone())).await;
                        }
                    }
                }
            },
        )
    }
}

#[derive(Debug, Clone)]
pub struct Logger(mpsc::UnboundedSender<LogMessage>);

impl Logger {
    pub fn log(&mut self, content: String, severity: Severity) {
        let _ = self
            .0
            .unbounded_send(LogMessage { content, severity })
            .unwrap();
    }
}

pub enum LoggerEvent {
    Initialized(Logger),
    Error(String),
}
