use iced::{Background, Color, Vector};

use crate::gui::{style::DEFAULT_BUTTON_BACKGROUND_COLOR, TrackerTheme};

#[derive(Default)]
pub enum ButtonStyle {
    #[default]
    Default,
    SetSelector,
}

impl iced::widget::button::StyleSheet for TrackerTheme {
    type Style = ButtonStyle;

    fn active(&self, style: &Self::Style) -> iced::widget::button::Appearance {
        match style {
            ButtonStyle::SetSelector => iced::widget::button::Appearance {
                shadow_offset: Vector::default(),
                background: None,
                border: iced::Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: iced::border::Radius::from(0.0),
                },
                text_color: Color::BLACK,
                ..Default::default()
            },
            ButtonStyle::Default => iced::widget::button::Appearance {
                shadow_offset: Vector::default(),
                background: Some(Background::Color(DEFAULT_BUTTON_BACKGROUND_COLOR)),
                border: iced::Border {
                    color: Color::BLACK,
                    width: 1.0,
                    radius: iced::border::Radius::from(2.0),
                },
                text_color: Color::BLACK,
                ..Default::default()
            },
        }
    }
}
