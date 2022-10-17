use iced::{
    pure::{container, scrollable, text, widget::Column, Element},
    Length,
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
        let messages: Element<TrackerMessage> = Column::with_children(
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
