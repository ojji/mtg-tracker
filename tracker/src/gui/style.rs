use iced::{container, Background, Color};

pub struct TooltipStyle;
impl container::StyleSheet for TooltipStyle {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(Color::from_rgb(0.9, 0.9, 0.9))),
            border_color: Color::BLACK,
            border_width: 1.0,
            border_radius: 2.0,
            ..Default::default()
        }
    }
}
