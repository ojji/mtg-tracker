use crate::gui::{style::BOX_BACKGROUND_COLOR, style::TOOLTIP_BACKGROUND_COLOR, TrackerTheme};
use iced::{widget::container::Appearance, Background, Color};

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
                border: iced::Border {
                    color: Color::BLACK,
                    width: 1.0,
                    radius: iced::border::Radius::from(2.0),
                },
                ..Default::default()
            },
            ContainerStyle::SummaryPanel => Appearance {
                border: iced::Border {
                    color: Color::BLACK,
                    width: 1.0,
                    radius: iced::border::Radius::default(),
                },
                ..Default::default()
            },
            ContainerStyle::Box => Appearance {
                text_color: None,
                background: Some(Background::Color(BOX_BACKGROUND_COLOR)),
                border: iced::Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: iced::border::Radius::from(2.0),
                },
                ..Default::default()
            },
        }
    }
}
