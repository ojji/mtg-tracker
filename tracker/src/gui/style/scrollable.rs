use iced::{
    widget::scrollable::{Scroller, StyleSheet},
    Background, Color,
};

use crate::gui::{
    style::{HOVERED_SCROLLER_COLOR, SCROLLABLE_BACKGROUND_COLOR, SCROLLER_COLOR},
    TrackerTheme,
};

#[derive(Default)]
pub enum ScrollableStyle {
    #[default]
    Default,
}

impl StyleSheet for TrackerTheme {
    type Style = ScrollableStyle;

    fn active(&self, _style: &Self::Style) -> iced::widget::scrollable::Appearance {
        iced::widget::scrollable::Appearance {
            container: iced::widget::container::Appearance::default(),
            scrollbar: iced::widget::scrollable::Scrollbar {
                background: Some(Background::Color(SCROLLABLE_BACKGROUND_COLOR)),
                border: iced::Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: iced::border::Radius::from(0.0),
                },
                scroller: Scroller {
                    color: SCROLLER_COLOR,
                    border: iced::Border {
                        color: SCROLLER_COLOR,
                        width: 0.0,
                        radius: iced::border::Radius::from(5.0),
                    },
                },
            },
            gap: None,
        }
    }

    fn hovered(
        &self,
        _style: &Self::Style,
        is_mouse_over_scrollbar: bool,
    ) -> iced::widget::scrollable::Appearance {
        if is_mouse_over_scrollbar {
            iced::widget::scrollable::Appearance {
                container: iced::widget::container::Appearance::default(),
                scrollbar: iced::widget::scrollable::Scrollbar {
                    background: Some(Background::Color(SCROLLABLE_BACKGROUND_COLOR)),
                    border: iced::Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: iced::border::Radius::from(0.0),
                    },
                    scroller: iced::widget::scrollable::Scroller {
                        color: HOVERED_SCROLLER_COLOR,
                        border: iced::Border {
                            color: HOVERED_SCROLLER_COLOR,
                            width: 0.0,
                            radius: iced::border::Radius::from(5.0),
                        },
                    },
                },
                gap: None,
            }
        } else {
            iced::widget::scrollable::Appearance {
                container: iced::widget::container::Appearance::default(),
                scrollbar: iced::widget::scrollable::Scrollbar {
                    background: Some(Background::Color(SCROLLABLE_BACKGROUND_COLOR)),
                    border: iced::Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: iced::border::Radius::from(0.0),
                    },
                    scroller: iced::widget::scrollable::Scroller {
                        color: SCROLLER_COLOR,
                        border: iced::Border {
                            color: SCROLLER_COLOR,
                            width: 0.0,
                            radius: iced::border::Radius::from(5.0),
                        },
                    },
                },
                gap: None,
            }
        }
    }
}
