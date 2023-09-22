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
                border_radius: 0.0.into(),
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
                text_color: Color::BLACK,
            },
            ButtonStyle::Default => iced::widget::button::Appearance {
                shadow_offset: Vector::default(),
                background: Some(Background::Color(DEFAULT_BUTTON_BACKGROUND_COLOR)),
                border_radius: 2.0.into(),
                border_width: 1.0,
                border_color: Color::BLACK,
                text_color: Color::BLACK,
            },
        }
    }
}
