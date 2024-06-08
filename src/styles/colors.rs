use iced::Color;

pub const NICE_GREEN: Color = Color::from_rgb(0.0, 0.835, 0.3);
pub const LIGHT_DARK_GREEN: Color = Color::from_rgb(0.0, 0.6, 0.212);

pub const LIGHT_GREY: Color = Color::from_rgb(0.75, 0.75, 0.75);
pub const GREY: Color = Color::from_rgb(0.5, 0.5, 0.5);
pub const DARK_GREY: Color = Color::from_rgb(0.25, 0.25, 0.25);

pub fn mix_color(a: Color, b: Color) -> Color {
	Color {
		r: (a.r + b.r) / 2.0,
		g: (a.g + b.g) / 2.0,
		b: (a.b + b.b) / 2.0,
		a: (a.a + b.a) / 2.0,
	}
}
