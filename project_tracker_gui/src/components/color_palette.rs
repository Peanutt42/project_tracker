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
			color_item(Color::BLACK),
			color_item(Color::WHITE),
			color_item(Color::from_rgb8(255, 0, 0)), // red
			color_item(Color::from_rgb8(0, 255, 0)), // green
			color_item(Color::from_rgb8(0, 0, 255)), // blue
			color_item(Color::from_rgb8(255, 127, 0)), // orange
			color_item(Color::from_rgb8(255, 230, 0)), // yellow
			color_item(Color::from_rgb8(255, 0, 255)), // pink
		]
	)
	.style(theme::Container::Custom(Box::new(PaletteContainerStyle)))
	.into()
}