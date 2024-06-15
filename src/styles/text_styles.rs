use iced::{theme, Color};
use crate::styles::NICE_GREEN;

pub const GREEN_TEXT_STYLE: theme::Text = theme::Text::Color(NICE_GREEN);
pub const DISABLED_GREEN_TEXT_STYLE: theme::Text = theme::Text::Color(Color{ a: 0.5, ..NICE_GREEN });
