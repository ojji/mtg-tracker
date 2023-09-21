use iced::{
    widget::{column, container, scrollable, text},
    Element, Length,
};

use crate::gui::TrackerMessage;

pub struct LogViewComponent {
    messages: Vec<String>,
}

impl LogViewComponent {
    pub fn new() -> LogViewComponent {
        LogViewComponent { messages: vec![] }
    }

    pub fn log_message(&mut self, message: String) {
        self.messages.insert(0, message);
    }

    pub fn view(&self) -> Element<TrackerMessage> {
        let messages: Element<TrackerMessage> = column(
            self.messages
                .iter()
                .map(|msg| text(msg).size(12).into())
                .collect::<Vec<Element<_>>>(),
        )
        .width(Length::Fill)
        .into();

        let scrollable_messages = scrollable(messages);

        container(scrollable_messages).into()
    }
}
