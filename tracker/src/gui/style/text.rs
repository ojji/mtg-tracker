use iced::widget::text::Appearance;

use crate::gui::{TrackerTheme, style::{DEFAULT_TEXT_COLOR, ERROR_TEXT_COLOR}};

#[derive(Default, Clone, Copy)]
pub enum TextStyle {
    #[default]
    Normal,
    Error,
}

impl iced::widget::text::StyleSheet for TrackerTheme {
    type Style = TextStyle;

    fn appearance(&self, style: Self::Style) -> Appearance {
        match style {
            TextStyle::Normal => Appearance {
                color: Some(DEFAULT_TEXT_COLOR),
            },
            TextStyle::Error => Appearance { color: Some(ERROR_TEXT_COLOR) },
        }
    }
}
