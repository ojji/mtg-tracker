pub mod button;
pub mod container;
pub mod scrollable;
pub mod text;

use iced::{
    application::{self, Appearance},
    Color,
};

const APP_BACKGROUND_COLOR: Color = Color::from_rgb(1.0, 1.0, 1.0);
const DEFAULT_TEXT_COLOR: Color = Color::from_rgb(0.0, 0.0, 0.0);
const ERROR_TEXT_COLOR: Color = Color::from_rgb(1.0, 0.0, 0.0);
const SCROLLER_COLOR: Color = Color::from_rgb(0.756, 0.756, 0.756);
const HOVERED_SCROLLER_COLOR: Color = Color::from_rgb(0.658, 0.658, 0.658);
const SCROLLABLE_BACKGROUND_COLOR: Color = Color::from_rgb(0.945, 0.945, 0.945);
const TOOLTIP_BACKGROUND_COLOR: Color = Color::from_rgb(0.9, 0.9, 0.9);
const DEFAULT_BUTTON_BACKGROUND_COLOR: Color = Color::from_rgb(0.87, 0.87, 0.87);
const BOX_BACKGROUND_COLOR: Color = Color::from_rgb(0.87, 0.87, 0.87);

#[derive(Default)]
pub enum TrackerThemeStyle {
    #[default]
    Default,
}

#[derive(Default)]
pub struct TrackerTheme {}

impl application::StyleSheet for TrackerTheme {
    type Style = TrackerThemeStyle;

    fn appearance(&self, _style: &Self::Style) -> application::Appearance {
        Appearance {
            background_color: APP_BACKGROUND_COLOR,
            text_color: DEFAULT_TEXT_COLOR,
        }
    }
}
