use iced::Color;

pub fn text_color(background: Color) -> Color {
	let brightness = 0.2126 * background.r + 0.7152 * background.g + 0.0722 * background.b;
	if brightness > 0.6 {
		Color::from_rgb(0.1, 0.1, 0.1)
	} else {
		Color::from_rgb(0.9, 0.9, 0.9)
	}
}
