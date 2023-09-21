use iced::{
    application::{self, Appearance},
    theme::Container,
    Background, BorderRadius, Color,
};

const APP_BACKGROUND_COLOR: Color = Color::from_rgb(1.0, 1.0, 1.0);
const DEFAULT_TEXT_COLOR: Color = Color::from_rgb(0.0, 0.0, 0.0);

#[derive(Default)]
pub enum TrackerThemeStyle {
    #[default]
    Default,
}

#[derive(Default)]
pub struct TrackerTheme {}

impl application::StyleSheet for TrackerTheme {
    type Style = TrackerThemeStyle;

    fn appearance(&self, style: &Self::Style) -> application::Appearance {
        Appearance {
            background_color: APP_BACKGROUND_COLOR,
            text_color: DEFAULT_TEXT_COLOR,
        }
    }
}

pub struct TooltipStyle;
impl iced::widget::container::StyleSheet for TooltipStyle {
    type Style = Container;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.9, 0.9, 0.9))),
            border_radius: BorderRadius::from(2.0),
            border_width: 1.0,
            border_color: Color::BLACK,
            ..Default::default()
        }
    }
    // fn style(&self) -> container::Style {
    //     container::Style {
    //         background: Some(Background::Color(Color::from_rgb(0.9, 0.9, 0.9))),
    //         border_color: Color::BLACK,
    //         border_width: 1.0,
    //         border_radius: 2.0,
    //         ..Default::default()
    //     }
    // }
}

// pub struct DraftSummaryPanel;
// impl iced::widget::container::StyleSheet for DraftSummaryPanel {
//     // fn style(&self) -> container::Style {
//     //     container::Style {
//     //         border_color: Color::BLACK,
//     //         border_width: 1.0,
//     //         ..Default::default()
//     //     }
//     // }
// }

// pub struct DebugStyle;
// impl iced::widget::container::StyleSheet for DebugStyle {
//     // fn style(&self) -> container::Style {
//     //     container::Style {
//     //         border_color: Color::BLACK,
//     //         border_width: 1.0,
//     //         ..Default::default()
//     //     }
//     // }
// }
