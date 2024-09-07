use iced::{theme, widget::{container, row}, Color, Element};
use crate::{project_tracker::UiMessage, styles::PaletteContainerStyle, components::color_palette_item_button};

pub const COLOR_PALETTE_BLACK: Color = Color {
	r: 0.04,
	g: 0.04,
	b: 0.04,
	a: 1.0
};

pub const COLOR_PALETTE_WHITE: Color = Color {
	r: 0.96,
	g: 0.96,
	b: 0.96,
	a: 1.0
};

pub fn color_palette(selected_color: Color, on_submit: impl Fn(Color) -> UiMessage) -> Element<'static, UiMessage> {
	let color_item = |color: Color| {
		color_palette_item_button(color, selected_color == color, on_submit(color))
	};

	container(
		row![
			color_item(COLOR_PALETTE_BLACK), // black
			color_item(COLOR_PALETTE_WHITE), // white
			color_item(Color::from_rgb8(255, 54, 6)), // red
			color_item(Color::from_rgb8(162, 250, 163)), // green
			color_item(Color::from_rgb8(154, 196, 248)), // blue
			color_item(Color::from_rgb8(245, 143, 41)), // orange
			color_item(Color::from_rgb8(255, 233, 49)), // yellow
			color_item(Color::from_rgb8(161, 79, 195)), // pink
		]
	)
	.style(theme::Container::Custom(Box::new(PaletteContainerStyle)))
	.into()
}