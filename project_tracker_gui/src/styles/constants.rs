pub mod border_radius {
	pub const BORDER_RADIUS: f32 = 5.0;
	pub const LARGE_BORDER_RADIUS: f32 = 7.5;
}

pub mod spacing {
	pub const TINY_SPACING_AMOUNT: f32 = 2.5;
	pub const SMALL_SPACING_AMOUNT: u16 = 5;
	pub const SPACING_AMOUNT: u16 = 10;
	pub const LARGE_SPACING_AMOUNT: u16 = 20;
}

pub mod size {
	pub const SMALL_TEXT_SIZE: f32 = 13.0;
	pub const LARGE_TEXT_SIZE: f32 = 20.0;
	pub const TITLE_TEXT_SIZE: f32 = 35.0;
}

pub mod colors {
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

	pub fn is_color_dark(color: Color) -> bool {
		(0.2126 * color.r + 0.7152 * color.g + 0.0722 * color.b) < 0.6
	}
}

pub mod padding {
	use iced::Padding;

	pub const SMALL_PADDING_AMOUNT: f32 = 5.0;
	pub const PADDING_AMOUNT: f32 = 10.0;
	pub const LARGE_PADDING_AMOUNT: f32 = 20.0;

	pub const HORIZONTAL_PADDING: Padding = Padding {
			left: PADDING_AMOUNT,
			right: PADDING_AMOUNT,
			..Padding::ZERO
	};

	pub const SMALL_HORIZONTAL_PADDING: Padding = Padding {
			left: SMALL_PADDING_AMOUNT,
			right: SMALL_PADDING_AMOUNT,
			..Padding::ZERO
	};
}