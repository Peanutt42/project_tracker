use iced::{theme, Color};
use crate::styles::NICE_GREEN;

pub const GREEN_TEXT_STYLE: theme::Text = theme::Text::Color(NICE_GREEN);
pub const DISABLED_GREEN_TEXT_STYLE: theme::Text = theme::Text::Color(Color{ a: 0.5, ..NICE_GREEN });

pub fn text_color(background: Color) -> Color {
	let brightness = 0.2126 * background.r + 0.7152 * background.g + 0.0722 * background.b;
	if brightness > 0.6 {
		Color::BLACK
	}
	else {
		Color::WHITE
	}
}

pub fn strikethrough_text(text: &str) -> String {
	let mut result = String::with_capacity(text.len() * 2);
	for char in text.chars() {
		result.push(char);
		result.push('\u{0336}'); // strikethrough: 'H̶e̶l̶l̶o̶,̶ ̶W̶o̶r̶l̶d̶!̶'
	}
	result
}

pub fn capped_text(text: &str, max_chars: usize) -> String {
	if text.len() <= max_chars {
		text.to_string()
	}
	else {
		// removes 3 chars for '...', while not gettings negativ
		let remaining_chars = max_chars.saturating_sub(3);
		format!("{}...", &text[..remaining_chars])
	}
}