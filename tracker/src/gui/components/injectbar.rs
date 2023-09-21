use crate::gui::{Action, TrackerMessage};
use iced::{
    alignment::Vertical,
    widget::{button, container, row, text},
    Alignment, Color, Element, Length,
};

pub struct InjectBarComponent {
    status: Status,
}

pub enum Status {
    Ok(String),
    Error(String),
}

impl InjectBarComponent {
    pub fn new() -> InjectBarComponent {
        InjectBarComponent {
            status: Status::Ok(String::from("Initializing...")),
        }
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = status;
    }

    pub fn view(&self) -> Element<TrackerMessage> {
        let inject_button =
            button(text("Inject")).on_press(TrackerMessage::Action(Action::Reinject));

        let toggle_button = button(text("Switch between views"))
            .on_press(TrackerMessage::Action(Action::SwitchView));

        let inject_status_text = match &self.status {
            Status::Ok(value) => text(value)
                .style(iced::theme::Text::Default)
                .vertical_alignment(Vertical::Center),
            Status::Error(e) => text(e)
                // .color([1.0, 0.0, 0.0])
                .style(iced::theme::Text::Color(Color::from_rgb(1.0, 0.0, 0.0)))
                .vertical_alignment(Vertical::Center),
        };

        let status_bar = row![
            inject_button,
            container(inject_status_text).width(Length::Fill).padding(5),
            toggle_button
        ]
        .align_items(Alignment::Center);

        container(status_bar).height(Length::Shrink).into()
    }
}
