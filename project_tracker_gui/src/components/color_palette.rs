use iced::{theme, widget::{button, Button, container, row}, Border, Color, Element, Length};
use iced_aw::{quad::Quad, widgets::InnerBounds};
use crate::{project_tracker::UiMessage, styles::{PaletteContainerStyle, ColorPaletteButtonStyle}};

pub fn color_palette_item_button(color: Color, selected: bool, on_press: UiMessage) -> Button<'static, UiMessage> {
	button(
		Quad {
			width: Length::Fixed(25.0),
			height: Length::Fixed(25.0),
			inner_bounds: InnerBounds::Ratio(0.8, 0.8),
			quad_color: color.into(),
			quad_border: Border::with_radius(f32::MAX),
			bg_color: None,
			..Default::default()
		}
	)
	.on_press(on_press)
	.style(theme::Button::custom(ColorPaletteButtonStyle{ selected }))
}

pub fn color_palette(selected_color: Color, on_submit: impl Fn(Color) -> UiMessage) -> Element<'static, UiMessage> {
	let color_item = |color: Color| {
		color_palette_item_button(color, selected_color == color, on_submit(color))
	};

	container(
		row![
			color_item(Color::from_rgb8(10, 10, 10)), // black
			color_item(Color::from_rgb8(245, 245, 245)), // white
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