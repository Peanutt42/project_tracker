use crate::{
	components::color_palette_item_button,
	project_tracker::Message,
	styles::{palette_container_style, GREY},
};
use iced::{
	widget::{column, container, row},
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
	let color_item = |color: Color,
	                  round_left_top: bool,
	                  round_right_top: bool,
	                  round_left_bottom: bool,
	                  round_right_bottom| {
		color_palette_item_button(
			color,
			selected_color == color,
			round_left_top,
			round_right_top,
			round_left_bottom,
			round_right_bottom,
			on_submit(color),
		)
	};

	container(column![
		row![
			color_item(COLOR_PALETTE_BLACK, true, false, false, false), // black
			color_item(COLOR_PALETTE_WHITE, false, false, false, false), // white
			color_item(Color::from_rgb8(255, 54, 6), false, false, false, false), // red
			color_item(Color::from_rgb8(245, 143, 41), false, false, false, false), // orange
			color_item(Color::from_rgb8(255, 233, 49), false, false, false, false), // yellow
			color_item(Color::from_rgb8(162, 250, 163), false, false, false, false), // green
			color_item(Color::from_rgb8(154, 196, 248), false, false, false, false), // blue
			color_item(Color::from_rgb8(255, 0, 144), false, true, false, false), // magenta
		],
		row![
			color_item(GREY, false, false, true, false), // grey
			color_item(Color::from_rgb8(175, 175, 175), false, false, false, false), // light grey
			color_item(Color::from_rgb8(167, 33, 0), false, false, false, false), // dark red
			color_item(Color::from_rgb8(139, 49, 0), false, false, false, false), // brown
			color_item(Color::from_rgb8(229, 156, 0), false, false, false, false), // dark yellow
			color_item(Color::from_rgb8(27, 135, 0), false, false, false, false), // dark green
			color_item(Color::from_rgb8(0, 37, 158), false, false, false, false), // dark blue
			color_item(Color::from_rgb8(161, 79, 195), false, false, false, true), // pink
		]
	])
	.style(palette_container_style)
	.into()
}
