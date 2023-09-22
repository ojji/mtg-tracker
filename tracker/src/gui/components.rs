pub mod setselector;
pub mod draftsummary;
pub mod injectbar;
pub mod logview;
pub mod collection;

use crate::gui::style::TrackerTheme;

pub type Element<'a, Message> = iced::Element<'a, Message, iced::Renderer<TrackerTheme>>;