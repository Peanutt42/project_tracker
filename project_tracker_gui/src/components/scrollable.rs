use crate::{
	project_tracker::Message,
	styles::{scrollable_style, SMALL_PADDING_AMOUNT},
};
use iced::{
	widget::{
		container, scrollable,
		scrollable::{Direction, Scrollbar},
		Scrollable,
	},
	Element, Padding,
};

pub const SCROLLBAR_WIDTH: f32 = SMALL_PADDING_AMOUNT;

pub const HORIZONTAL_SCROLLABLE_PADDING: Padding = Padding {
	bottom: SCROLLBAR_WIDTH + SMALL_PADDING_AMOUNT,
	..Padding::ZERO
};

pub fn horizontal_scrollable<'a, Message: 'a>(
	content: impl Into<Element<'a, Message>>,
) -> Scrollable<'a, Message> {
	scrollable(container(content).padding(Padding {
		bottom: SCROLLBAR_WIDTH + SMALL_PADDING_AMOUNT,
		..Padding::ZERO
	}))
	.direction(Direction::Horizontal(
		Scrollbar::new().scroller_width(SCROLLBAR_WIDTH),
	))
	.style(scrollable_style)
}

pub fn vertical_scrollable<'a>(
	content: impl Into<Element<'a, Message>>,
) -> Scrollable<'a, Message> {
	scrollable(container(content).padding(Padding {
		left: SCROLLBAR_WIDTH + SMALL_PADDING_AMOUNT,
		right: SCROLLBAR_WIDTH + SMALL_PADDING_AMOUNT,
		..Padding::ZERO
	}))
	.direction(Direction::Vertical(
		Scrollbar::new().scroller_width(SCROLLBAR_WIDTH),
	))
	.style(scrollable_style)
}
