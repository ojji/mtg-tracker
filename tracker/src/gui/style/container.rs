use crate::gui::{style::TOOLTIP_BACKGROUND_COLOR, style::BOX_BACKGROUND_COLOR, TrackerTheme};
use iced::{widget::container::Appearance, Background, BorderRadius, Color};

#[derive(Default)]
pub enum ContainerStyle {
    #[default]
    Default,
    Tooltip,
    SummaryPanel,
    Box,
}

impl iced::widget::container::StyleSheet for TrackerTheme {
    type Style = ContainerStyle;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        match style {
            ContainerStyle::Default => Appearance::default(),
            ContainerStyle::Tooltip => Appearance {
                background: Some(Background::Color(TOOLTIP_BACKGROUND_COLOR)),
                border_radius: BorderRadius::from(2.0),
                border_width: 1.0,
                border_color: Color::BLACK,
                ..Default::default()
            },
            ContainerStyle::SummaryPanel => Appearance {
                border_width: 1.0,
                border_color: Color::BLACK,
                ..Default::default()
            },
            ContainerStyle::Box => {
                Appearance {
                    text_color: None,
                    background: Some(Background::Color(BOX_BACKGROUND_COLOR)),
                    border_radius: 2.0.into(),
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                }
            },
        }
    }
}
