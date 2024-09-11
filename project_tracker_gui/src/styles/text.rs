use iced::Color;

pub fn text_color(background: Color) -> Color {
	let brightness = 0.2126 * background.r + 0.7152 * background.g + 0.0722 * background.b;
	if brightness > 0.6 {
		Color::from_rgb(0.1, 0.1, 0.1)
	}
	else {
		Color::from_rgb(0.9, 0.9, 0.9)
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