use crate::{
	components::color_palette_item_button, project_tracker::Message,
	styles::palette_container_style,
};
use iced::{
	widget::{container, row},
	Color, Element,
};

pub const COLOR_PALETTE_BLACK: Color = Color {
	r: 10.0 / 255.0,
	g: 10.0 / 255.0,
	b: 10.0 / 255.0,
	a: 1.0,
};

pub const COLOR_PALETTE_WHITE: Color = Color {
	r: 245.0 / 255.0,
	g: 245.0 / 255.0,
	b: 245.0 / 255.0,
	a: 1.0,
};

pub fn color_palette(
	selected_color: Color,
	on_submit: impl Fn(Color) -> Message,
) -> Element<'static, Message> {
	let color_item = |color: Color, round_left: bool, round_right: bool| {
		color_palette_item_button(
			color,
			selected_color == color,
			round_left,
			round_right,
			on_submit(color),
		)
	};

	container(row![
		color_item(COLOR_PALETTE_BLACK, true, false),  // black
		color_item(COLOR_PALETTE_WHITE, false, false), // white
		color_item(Color::from_rgb8(255, 54, 6), false, false), // red
		color_item(Color::from_rgb8(162, 250, 163), false, false), // green
		color_item(Color::from_rgb8(154, 196, 248), false, false), // blue
		color_item(Color::from_rgb8(245, 143, 41), false, false), // orange
		color_item(Color::from_rgb8(255, 233, 49), false, false), // yellow
		color_item(Color::from_rgb8(161, 79, 195), false, true), // pink
	])
	.style(palette_container_style)
	.into()
}
