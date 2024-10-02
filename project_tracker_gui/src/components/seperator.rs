use crate::{
	project_tracker::UiMessage,
	styles::{GREY, PADDING_AMOUNT},
};
use iced::{
	widget::container,
	Color, Element,
	Length::{self, Fill},
	Padding,
};
use iced_aw::{quad::Quad, widgets::InnerBounds};

pub fn horizontal_seperator_padded() -> Element<'static, UiMessage> {
	container(horizontal_seperator())
		.padding(Padding {
			top: PADDING_AMOUNT,
			bottom: PADDING_AMOUNT,
			..Padding::ZERO
		})
		.into()
}

pub fn horizontal_seperator() -> Quad {
	horizontal_seperator_colored(GREY)
}

pub fn horizontal_seperator_colored(color: Color) -> Quad {
	Quad {
		width: Fill,
		height: Length::Fixed(1.0),
		inner_bounds: InnerBounds::Ratio(1.0, 1.0),
		quad_color: color.into(),
		..Default::default()
	}
}

pub fn vertical_seperator() -> Quad {
	Quad {
		width: Length::Fixed(1.0),
		height: Fill,
		inner_bounds: InnerBounds::Ratio(1.0, 1.0),
		quad_color: GREY.into(),
		..Default::default()
	}
}
