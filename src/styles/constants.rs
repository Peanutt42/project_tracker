pub mod border_radius {
	pub const BORDER_RADIUS: f32 = 5.0;
	pub const LARGE_BORDER_RADIUS: f32 = 7.5;
}

pub mod blur_radius {
	pub const BLUR_RADIUS: f32 = 15.0;
	pub const LARGE_BLUR_RADIUS: f32 = 25.0;
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
	pub const HEADING_TEXT_SIZE: f32 = 25.0;
	pub const TITLE_TEXT_SIZE: f32 = 35.0;

	pub const MINIMAL_DRAG_DISTANCE: f32 = 10.0;
}

pub mod colors {
	use iced::{color, theme::palette, Color};

	pub const GREY: Color = Color::from_rgb(0.5, 0.5, 0.5);

	pub fn selection_color(palette: &palette::Extended) -> Color {
		if palette.is_dark {
			color!(0x0000ee)
		} else {
			color!(0x3584e4)
		}
	}

	pub fn link_color(is_theme_dark: bool) -> Color {
		if is_theme_dark {
			iced::color!(0x3584e4)
		} else {
			iced::color!(0x0000ee)
		}
	}

	pub fn background_shadow_color(palette: &palette::Extended) -> Color {
		let background = palette.background.base.color;
		let amount = if palette.is_dark { 0.1 } else { 0.15 };

		Color {
			r: background.r - amount,
			g: background.g - amount,
			b: background.b - amount,
			a: background.a,
		}
	}

	pub fn color_average(a: Color, b: Color) -> Color {
		Color {
			r: (a.r + b.r) / 2.0,
			g: (a.g + b.g) / 2.0,
			b: (a.b + b.b) / 2.0,
			a: (a.a + b.a) / 2.0,
		}
	}

	pub fn mix_color(a: Color, b: Color, factor: f32) -> Color {
		Color {
			r: a.r + (b.r - a.r) * factor,
			g: a.g + (b.g - a.g) * factor,
			b: a.b + (b.b - a.b) * factor,
			a: a.a + (b.a - a.a) * factor,
		}
	}
}

pub mod padding {
	use iced::Padding;

	pub const SMALL_PADDING_AMOUNT: f32 = 5.0;
	pub const PADDING_AMOUNT: f32 = 10.0;
	pub const LARGE_PADDING_AMOUNT: f32 = 20.0;

	pub const SMALL_HORIZONTAL_PADDING: Padding = Padding {
		left: SMALL_PADDING_AMOUNT,
		right: SMALL_PADDING_AMOUNT,
		..Padding::ZERO
	};
}

pub const GAP: f32 = 5.0;
