use iced::{
    widget::scrollable::{Scrollbar, Scroller, StyleSheet},
    Background, BorderRadius, Color,
};

use crate::gui::{
    style::{SCROLLABLE_BACKGROUND_COLOR, HOVERED_SCROLLER_COLOR, SCROLLER_COLOR},
    TrackerTheme,
};

#[derive(Default)]
pub enum ScrollableStyle {
    #[default]
    Default,
}

impl StyleSheet for TrackerTheme {
    type Style = ScrollableStyle;

    fn active(&self, _style: &Self::Style) -> Scrollbar {
        iced::widget::scrollable::Scrollbar {
            border_color: Color::TRANSPARENT,
            background: Some(Background::Color(SCROLLABLE_BACKGROUND_COLOR)),
            border_radius: BorderRadius::from(0.0),
            border_width: 0.0,
            scroller: Scroller {
                border_width: 0.0,
                border_radius: BorderRadius::from(5.0),
                border_color: SCROLLER_COLOR,
                color: SCROLLER_COLOR,
            },
        }
    }

    fn hovered(
        &self,
        _style: &Self::Style,
        is_mouse_over_scrollbar: bool,
    ) -> iced::widget::scrollable::Scrollbar {
        if is_mouse_over_scrollbar {
            iced::widget::scrollable::Scrollbar {
                border_color: Color::TRANSPARENT,
                background: Some(Background::Color(SCROLLABLE_BACKGROUND_COLOR)),
                border_radius: BorderRadius::from(0.0),
                border_width: 0.0,
                scroller: Scroller {
                    border_width: 0.0,
                    border_radius: BorderRadius::from(5.0),
                    border_color: HOVERED_SCROLLER_COLOR,
                    color: HOVERED_SCROLLER_COLOR,
                },
            }
        } else {
            iced::widget::scrollable::Scrollbar {
                border_color: Color::TRANSPARENT,
                background: Some(Background::Color(SCROLLABLE_BACKGROUND_COLOR)),
                border_radius: BorderRadius::from(0.0),
                border_width: 0.0,
                scroller: Scroller {
                    border_width: 0.0,
                    border_radius: BorderRadius::from(5.0),
                    border_color: SCROLLER_COLOR,
                    color: SCROLLER_COLOR,
                },
            }
        }
    }
}
