use iced::{theme, widget::{scrollable, container, scrollable::{Direction, Properties}, Scrollable}, Element, Padding};
use crate::{project_tracker::UiMessage, styles::{ScrollableStyle, SMALL_PADDING_AMOUNT}};

pub const SCROLLBAR_WIDTH: f32 = SMALL_PADDING_AMOUNT;

pub const HORIZONTAL_SCROLLABLE_PADDING: Padding = Padding {
	bottom: SCROLLBAR_WIDTH + SMALL_PADDING_AMOUNT,
	..Padding::ZERO
};

pub fn horizontal_scrollable<'a>(content: impl Into<Element<'a, UiMessage>>) -> Scrollable<'a, UiMessage> {
	scrollable(
		container(
			content
		)
		.padding(Padding{
			bottom: SCROLLBAR_WIDTH + SMALL_PADDING_AMOUNT,
			..Padding::ZERO
		})
	)
	.direction(Direction::Horizontal(
		Properties::new().scroller_width(SCROLLBAR_WIDTH)
	))
	.style(theme::Scrollable::custom(ScrollableStyle))
}

pub fn vertical_scrollable<'a>(content: impl Into<Element<'a, UiMessage>>) -> Scrollable<'a, UiMessage> {
	scrollable(
		container(
			content
		)
		.padding(Padding {
			left: SCROLLBAR_WIDTH + SMALL_PADDING_AMOUNT,
			right: SCROLLBAR_WIDTH + SMALL_PADDING_AMOUNT,
			..Padding::ZERO
		})
	)
	.direction(Direction::Vertical(
		Properties::new().scroller_width(SCROLLBAR_WIDTH)
	))
	.style(theme::Scrollable::custom(ScrollableStyle))
}