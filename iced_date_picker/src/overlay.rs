//! Helper functions for overlays

use iced::{
	advanced::layout,
	keyboard::{key::Named, Key},
	Point, Size,
};

/// Trait containing functions for positioning of nodes.
pub trait Position {
	/// Centers this node around the given position. If the node is over the
	/// specified bounds it's bouncing back to be fully visible on screen.
	fn center_and_bounce(&mut self, position: Point, bounds: Size);
}

impl Position for layout::Node {
	fn center_and_bounce(&mut self, position: Point, bounds: Size) {
		let size = self.size();

		self.move_to_mut(Point::new(
			(position.x - (size.width / 2.0)).max(0.0),
			(position.y - (size.height / 2.0)).max(0.0),
		));

		let new_self_bounds = self.bounds();

		self.move_to_mut(Point::new(
			if new_self_bounds.x + new_self_bounds.width > bounds.width {
				(new_self_bounds.x - (new_self_bounds.width - (bounds.width - new_self_bounds.x)))
					.max(0.0)
			} else {
				new_self_bounds.x
			},
			if new_self_bounds.y + new_self_bounds.height > bounds.height {
				(new_self_bounds.y - (new_self_bounds.height - (bounds.height - new_self_bounds.y)))
					.max(0.0)
			} else {
				new_self_bounds.y
			},
		));
	}
}

use crate::{
	date::{Date, IsInMonth},
	date_picker,
	style::{Status, Style, StyleState},
};
use chrono::{Datelike, Local, NaiveDate};
use iced::{
	advanced::{
		layout::{Limits, Node},
		overlay, renderer,
		text::Renderer as _,
		widget::tree::Tree,
		Clipboard, Layout, Overlay, Renderer as _, Shell, Widget,
	},
	alignment::{Horizontal, Vertical},
	event,
	keyboard,
	mouse::{self, Cursor},
	touch,
	widget::{
		text::{self, Wrapping},
		Button, Column, Container, Row, Text,
	},
	Alignment,
	Border,
	Color,
	Element,
	Event,
	Length,
	Padding,
	Pixels,
	Rectangle,
	Renderer, // the actual type
	Shadow,
};
use iced_fonts::{
	required::{icon_to_string, RequiredIcons},
	REQUIRED_FONT,
};
use std::{collections::HashMap, rc::Rc};

/// The padding around the elements.
const PADDING: Padding = Padding::new(10.0);
/// The spacing between the elements.
const SPACING: Pixels = Pixels(15.0);
/// The padding of the day cells.
const DAY_CELL_PADDING: Padding = Padding::new(7.0);

/// The overlay of the [`DatePicker`](crate::widget::DatePicker).
#[allow(missing_debug_implementations)]
pub struct DatePickerOverlay<'a, 'b, Message, Theme>
where
	Message: Clone,
	Theme: crate::style::Catalog + iced::widget::button::Catalog,
	'b: 'a,
{
	/// The state of the [`DatePickerOverlay`].
	state: &'a mut State<Message>,
	/// The position of the [`DatePickerOverlay`].
	position: Point,
	/// The style of the [`DatePickerOverlay`].
	class: &'a <Theme as crate::style::Catalog>::Class<'b>,
	/// The reference to the tree holding the state of this overlay.
	tree: &'a mut Tree,
	/// The font size of text and icons in the [`DatePickerOverlay`]
	font_size: Pixels,
}

impl<'a, 'b, Message, Theme> DatePickerOverlay<'a, 'b, Message, Theme>
where
	Message: 'static + Clone,
	Theme: 'a
		+ crate::style::Catalog
		+ iced::widget::button::Catalog
		+ iced::widget::text::Catalog
		+ iced::widget::container::Catalog,
	'b: 'a,
{
	/// Creates a new [`DatePickerOverlay`] on the given position.
	pub fn new(
		state: &'a mut date_picker::State<Message>,
		position: Point,
		class: &'a <Theme as crate::style::Catalog>::Class<'b>,
		tree: &'a mut Tree,
		//button_style: impl Clone +  Into<<Renderer as button::Renderer>::Style>, // clone not satisfied
		font_size: Pixels,
	) -> Self {
		let date_picker::State { overlay_state } = state;

		DatePickerOverlay {
			state: overlay_state,
			position,
			class,
			tree,
			font_size,
		}
	}

	/// Turn this [`DatePickerOverlay`] into an overlay [`Element`](overlay::Element).
	#[must_use]
	pub fn overlay(self) -> overlay::Element<'a, Message, Theme, Renderer> {
		overlay::Element::new(Box::new(self))
	}

	/// String representation of the current month and year.
	fn date_as_string(&self) -> String {
		crate::date::date_as_string(self.state.date)
	}

	/// The event handling for the month / year bar.
	fn on_event_month_year(
		&mut self,
		event: &Event,
		layout: Layout<'_>,
		cursor: Cursor,
	) -> event::Status {
		let mut children = layout.children();

		let mut status = event::Status::Ignored;

		// ----------- Month ----------------------
		let month_layout = children
			.next()
			.expect("widget: Layout should have a month layout");
		let mut month_children = month_layout.children();

		let left_bounds = month_children
			.next()
			.expect("widget: Layout should have a left month arrow layout")
			.bounds();
		let _center_bounds = month_children
			.next()
			.expect("widget: Layout should have a center month layout")
			.bounds();
		let right_bounds = month_children
			.next()
			.expect("widget: Layout should have a right month arrow layout")
			.bounds();

		match event {
			Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
			| Event::Touch(touch::Event::FingerPressed { .. }) => {
				if cursor.is_over(month_layout.bounds()) {
					self.state.focus = Focus::Month;
				}

				if cursor.is_over(left_bounds) {
					self.state.date = crate::date::pred_month(self.state.date);
					status = event::Status::Captured;
				} else if cursor.is_over(right_bounds) {
					self.state.date = crate::date::succ_month(self.state.date);
					status = event::Status::Captured;
				}
			}
			_ => {}
		}

		// ----------- Year -----------------------
		let year_layout = children
			.next()
			.expect("widget: Layout should have a year layout");
		let mut year_children = year_layout.children();

		let left_bounds = year_children
			.next()
			.expect("widget: Layout should have a left year arrow layout")
			.bounds();
		let _center_bounds = year_children
			.next()
			.expect("widget: Layout should have a center year layout")
			.bounds();
		let right_bounds = year_children
			.next()
			.expect("widget: Layout should have a right year arrow layout")
			.bounds();

		match event {
			Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
			| Event::Touch(touch::Event::FingerPressed { .. }) => {
				if cursor.is_over(year_layout.bounds()) {
					self.state.focus = Focus::Year;
				}

				if cursor.is_over(left_bounds) {
					self.state.date = crate::date::pred_year(self.state.date);
					status = event::Status::Captured;
				} else if cursor.is_over(right_bounds) {
					self.state.date = crate::date::succ_year(self.state.date);
					status = event::Status::Captured;
				}
			}
			_ => {}
		}

		status
	}

	/// The event handling for the calendar days.
	fn on_event_days(
		&mut self,
		event: &Event,
		layout: Layout<'_>,
		cursor: Cursor,
		shell: &mut Shell<Message>,
	) -> event::Status {
		let mut children = layout.children();

		let _day_labels_layout = children
			.next()
			.expect("widget: Layout should have a day label layout");

		let mut status = event::Status::Ignored;

		match event {
			Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
			| Event::Touch(touch::Event::FingerPressed { .. }) => {
				if cursor.is_over(layout.bounds()) {
					self.state.focus = Focus::Day;
				}

				'outer: for (y, row) in children.enumerate() {
					for (x, label) in row.children().enumerate() {
						let bounds = label.bounds();
						if cursor.is_over(bounds) {
							let (day, is_in_month) = crate::date::position_to_day(
								x,
								y,
								self.state.date.year(),
								self.state.date.month(),
							);

							self.state.date = match is_in_month {
								IsInMonth::Previous => crate::date::pred_month(self.state.date)
									.with_day(day as u32)
									.expect("Previous month with day should be valid"),
								IsInMonth::Same => self
									.state
									.date
									.with_day(day as u32)
									.expect("Same month with day should be valid"),
								IsInMonth::Next => crate::date::succ_month(self.state.date)
									.with_day(day as u32)
									.expect("Succeeding month with day should be valid"),
							};
							if let Some(on_submit) = self.state.on_submit.as_ref() {
								shell.publish((on_submit)(self.state.date.into()));
							}

							status = event::Status::Captured;
							break 'outer;
						}
					}
				}
			}
			Event::Keyboard(keyboard::Event::KeyPressed {
				key: Key::Named(Named::Enter),
				..
			}) => {
				if let Some(on_submit) = self.state.on_submit.as_ref() {
					shell.publish((on_submit)(self.state.date.into()));
				}

				status = event::Status::Captured;
			}
			Event::Keyboard(keyboard::Event::KeyPressed {
				key: Key::Named(Named::Escape),
				..
			}) => {
				if let Some(on_close) = self.state.on_close.as_ref() {
					shell.publish(on_close.clone());
				}
				status = event::Status::Captured;
			}
			_ => {}
		}

		status
	}

	/// The event handling for the keyboard input.
	fn on_event_keyboard(&mut self, event: &Event) -> event::Status {
		if self.state.focus == Focus::None {
			return event::Status::Ignored;
		}

		if let Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) = event {
			let mut status = event::Status::Ignored;

			match key.as_ref() {
				keyboard::Key::Named(keyboard::key::Named::Tab) => {
					if self.state.keyboard_modifiers.shift() {
						self.state.focus = self.state.focus.previous();
					} else {
						self.state.focus = self.state.focus.next();
					}
				}
				keyboard::Key::Named(k) => match self.state.focus {
					Focus::Month => match k {
						keyboard::key::Named::ArrowLeft => {
							self.state.date = crate::date::pred_month(self.state.date);
							status = event::Status::Captured;
						}
						keyboard::key::Named::ArrowRight => {
							self.state.date = crate::date::succ_month(self.state.date);
							status = event::Status::Captured;
						}
						_ => {}
					},
					Focus::Year => match k {
						keyboard::key::Named::ArrowLeft => {
							self.state.date = crate::date::pred_year(self.state.date);
							status = event::Status::Captured;
						}
						keyboard::key::Named::ArrowRight => {
							self.state.date = crate::date::succ_year(self.state.date);
							status = event::Status::Captured;
						}
						_ => {}
					},
					Focus::Day => match k {
						keyboard::key::Named::ArrowLeft => {
							self.state.date = crate::date::pred_day(self.state.date);
							status = event::Status::Captured;
						}
						keyboard::key::Named::ArrowRight => {
							self.state.date = crate::date::succ_day(self.state.date);
							status = event::Status::Captured;
						}
						keyboard::key::Named::ArrowUp => {
							self.state.date = crate::date::pred_week(self.state.date);
							status = event::Status::Captured;
						}
						keyboard::key::Named::ArrowDown => {
							self.state.date = crate::date::succ_week(self.state.date);
							status = event::Status::Captured;
						}
						_ => {}
					},
					_ => {}
				},
				_ => {}
			}

			status
		} else if let Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) = event {
			self.state.keyboard_modifiers = *modifiers;
			event::Status::Ignored
		} else {
			event::Status::Ignored
		}
	}
}

impl<'a, 'b, Message, Theme> Overlay<Message, Theme, Renderer>
	for DatePickerOverlay<'a, 'b, Message, Theme>
where
	Message: 'static + Clone,
	Theme: 'a
		+ crate::style::Catalog
		+ iced::widget::button::Catalog
		+ iced::widget::text::Catalog
		+ iced::widget::container::Catalog,
	'b: 'a,
{
	#[allow(clippy::too_many_lines)]
	fn layout(&mut self, renderer: &Renderer, bounds: Size) -> Node {
		let limits = Limits::new(Size::ZERO, bounds)
			.shrink(PADDING)
			.width(Length::Shrink)
			.height(Length::Shrink);

		let limits = limits.shrink(Size::new(0.0, SPACING.0));

		// Month/Year
		let font_size = self.font_size;

		let month_year = Row::<Message, Theme, Renderer>::new()
			.width(Length::Shrink)
			.spacing(SPACING)
			.push(
				Row::new()
					.width(Length::Shrink)
					.spacing(SPACING)
					.align_y(Alignment::Center)
					.push(
						// Left Month arrow
						Container::new(
							Text::new(icon_to_string(RequiredIcons::CaretLeftFill))
								.size(font_size.0 + 1.0)
								.font(REQUIRED_FONT),
						)
						.height(Length::Shrink)
						.width(Length::Shrink),
					)
					.push(
						// Month
						Text::new("September").size(font_size).width(Length::Shrink),
					)
					.push(
						// Right Month arrow
						Container::new(
							Text::new(icon_to_string(RequiredIcons::CaretRightFill))
								.size(font_size.0 + 1.0)
								.font(REQUIRED_FONT),
						)
						.height(Length::Shrink)
						.width(Length::Shrink),
					),
			)
			.push(
				Row::new()
					.width(Length::Shrink)
					.spacing(SPACING)
					.align_y(Alignment::Center)
					.push(
						// Left Year arrow
						Container::new(
							Text::new(icon_to_string(RequiredIcons::CaretLeftFill))
								.size(font_size.0 + 1.0)
								.font(REQUIRED_FONT),
						)
						.height(Length::Shrink)
						.width(Length::Shrink),
					)
					.push(
						// Year
						Text::new("9999").size(font_size).width(Length::Shrink),
					)
					.push(
						// Right Year arrow
						Container::new(
							Text::new(icon_to_string(RequiredIcons::CaretRightFill))
								.size(font_size.0 + 1.0)
								.font(REQUIRED_FONT),
						)
						.height(Length::Shrink)
						.width(Length::Shrink),
					),
			);

		let days = Container::<Message, Theme, Renderer>::new((0..7).fold(
			Column::new().width(Length::Shrink).height(Length::Shrink),
			|column, _y| {
				column.push(
					(0..7).fold(
						Row::new()
							.height(Length::Shrink)
							.width(Length::Shrink)
							.spacing(SPACING),
						|row, _x| {
							row.push(
								Container::new(Row::new().push(Text::new("31").size(font_size)))
									.width(Length::Shrink)
									.height(Length::Shrink)
									.padding(DAY_CELL_PADDING),
							)
						},
					),
				)
			},
		))
		.width(Length::Shrink)
		.height(Length::Shrink)
		.center_y(Length::Shrink);

		let col = Column::<Message, Theme, Renderer>::new()
			.spacing(SPACING)
			.align_x(Alignment::Center)
			.push(month_year)
			.push(days);

		let element: Element<Message, Theme, Renderer> = Element::new(col);
		let col_tree = if let Some(child_tree) = self.tree.children.get_mut(2) {
			child_tree.diff(element.as_widget());
			child_tree
		} else {
			let child_tree = Tree::new(element.as_widget());
			self.tree.children.insert(2, child_tree);
			&mut self.tree.children[2]
		};

		let mut col = element.as_widget().layout(col_tree, renderer, &limits);
		let col_bounds = col.bounds();
		col = col.move_to(Point::new(
			col_bounds.x + PADDING.left,
			col_bounds.y + PADDING.top,
		));

		let mut node = Node::with_children(
			Size::new(
				col.bounds().width + PADDING.horizontal(),
				col.bounds().height + PADDING.vertical(),
			),
			vec![col],
		);
		node.center_and_bounce(self.position, bounds);
		node
	}

	fn on_event(
		&mut self,
		event: Event,
		layout: Layout<'_>,
		cursor: Cursor,
		_renderer: &Renderer,
		_clipboard: &mut dyn Clipboard,
		shell: &mut Shell<Message>,
	) -> event::Status {
		if event::Status::Captured == self.on_event_keyboard(&event) {
			return event::Status::Captured;
		}

		let mut children = layout.children();

		let mut date_children = children
			.next()
			.expect("widget: Layout should have date children")
			.children();

		// ----------- Year/Month----------------------
		let month_year_layout = date_children
			.next()
			.expect("widget: Layout should have a month/year layout");
		let month_year_status = self.on_event_month_year(&event, month_year_layout, cursor);

		// ----------- Days ----------------------
		let days_layout = date_children
			.next()
			.expect("widget: Layout should have a days table parent")
			.children()
			.next()
			.expect("widget: Layout should have a days table layout");
		let days_status = self.on_event_days(&event, days_layout, cursor, shell);

		month_year_status.merge(days_status)
	}

	fn mouse_interaction(
		&self,
		layout: Layout<'_>,
		cursor: Cursor,
		_viewport: &Rectangle,
		_renderer: &Renderer,
	) -> mouse::Interaction {
		let mouse_interaction = mouse::Interaction::default();

		let mut children = layout.children();
		let mut date_children = children
			.next()
			.expect("Graphics: Layout should have a date layout")
			.children();

		// Month and year mouse interaction
		let month_year_layout = date_children
			.next()
			.expect("Graphics: Layout should have a month/year layout");
		let mut month_year_children = month_year_layout.children();
		let month_layout = month_year_children
			.next()
			.expect("Graphics: Layout should have a month layout");
		let year_layout = month_year_children
			.next()
			.expect("Graphics: Layout should have a year layout");

		let f = |layout: Layout<'_>| {
			let mut children = layout.children();

			let left_bounds = children
				.next()
				.expect("Graphics: Layout should have a left arrow layout")
				.bounds();
			let _center = children.next();
			let right_bounds = children
				.next()
				.expect("Graphics: Layout should have a right arrow layout")
				.bounds();

			let mut mouse_interaction = mouse::Interaction::default();

			let left_arrow_hovered = cursor.is_over(left_bounds);
			let right_arrow_hovered = cursor.is_over(right_bounds);

			if left_arrow_hovered || right_arrow_hovered {
				mouse_interaction = mouse_interaction.max(mouse::Interaction::Pointer);
			}

			mouse_interaction
		};

		let month_mouse_interaction = f(month_layout);
		let year_mouse_interaction = f(year_layout);

		// Days
		let days_layout = date_children
			.next()
			.expect("Graphics: Layout should have a days layout parent")
			.children()
			.next()
			.expect("Graphics: Layout should have a days layout");
		let mut days_children = days_layout.children();
		let _day_labels_layout = days_children.next();

		let mut table_mouse_interaction = mouse::Interaction::default();

		for row in days_children {
			for label in row.children() {
				let bounds = label.bounds();

				let mouse_over = cursor.is_over(bounds);
				if mouse_over {
					table_mouse_interaction =
						table_mouse_interaction.max(mouse::Interaction::Pointer);
				}
			}
		}

		mouse_interaction
			.max(month_mouse_interaction)
			.max(year_mouse_interaction)
			.max(table_mouse_interaction)
	}

	fn draw(
		&self,
		renderer: &mut Renderer,
		theme: &Theme,
		_style: &renderer::Style,
		layout: Layout<'_>,
		cursor: Cursor,
	) {
		let bounds = layout.bounds();
		let mut children = layout.children();
		let mut date_children = children
			.next()
			.expect("Graphics: Layout should have a date layout")
			.children();

		let mut style_sheet: HashMap<StyleState, Style> = HashMap::new();
		let _ = style_sheet.insert(
			StyleState::Active,
			crate::style::Catalog::style(theme, self.class, Status::Active),
		);
		let _ = style_sheet.insert(
			StyleState::Selected,
			crate::style::Catalog::style(theme, self.class, Status::Selected),
		);
		let _ = style_sheet.insert(
			StyleState::Hovered,
			crate::style::Catalog::style(theme, self.class, Status::Hovered),
		);
		let _ = style_sheet.insert(
			StyleState::Focused,
			crate::style::Catalog::style(theme, self.class, Status::Focused),
		);

		let mut style_state = StyleState::Active;
		if self.state.focus == Focus::Overlay {
			style_state = style_state.max(StyleState::Focused);
		}
		if cursor.is_over(bounds) {
			style_state = style_state.max(StyleState::Hovered);
		}

		// Background
		if (bounds.width > 0.) && (bounds.height > 0.) {
			renderer.fill_quad(
				renderer::Quad {
					bounds,
					border: Border {
						radius: style_sheet[&style_state].border_radius.into(),
						width: style_sheet[&style_state].border_width,
						color: style_sheet[&style_state].border_color,
					},
					shadow: Shadow::default(),
				},
				style_sheet[&style_state].background,
			);
		}

		// ----------- Year/Month----------------------
		let month_year_layout = date_children
			.next()
			.expect("Graphics: Layout should have a month/year layout");

		month_year(
			renderer,
			month_year_layout,
			&self.date_as_string(),
			cursor.position().unwrap_or_default(),
			&style_sheet,
			self.state.focus,
			self.font_size,
		);

		// ----------- Days ---------------------------
		let days_layout = date_children
			.next()
			.expect("Graphics: Layout should have a days layout parent")
			.children()
			.next()
			.expect("Graphics: Layout should have a days layout");

		days(
			renderer,
			days_layout,
			self.state.date,
			cursor.position().unwrap_or_default(),
			&style_sheet,
			self.state.focus,
			self.font_size,
		);
	}
}

/// The state of the [`DatePickerOverlay`].
pub struct State<Message> {
	/// The selected date of the [`DatePickerOverlay`].
	pub(crate) date: NaiveDate,
	pub(crate) on_submit: Option<Rc<dyn Fn(Date) -> Message>>,
	pub(crate) on_close: Option<Message>,
	/// The focus of the [`DatePickerOverlay`].
	pub(crate) focus: Focus,
	/// The previously pressed keyboard modifiers.
	pub(crate) keyboard_modifiers: keyboard::Modifiers,
}

impl<Message> State<Message> {
	/// Creates a new State with the given date.
	#[must_use]
	pub fn new(date: NaiveDate, on_submit: Rc<dyn Fn(Date) -> Message>, on_close: Message) -> Self {
		Self {
			date,
			on_submit: Some(on_submit),
			on_close: Some(on_close),
			focus: Focus::default(),
			keyboard_modifiers: keyboard::Modifiers::default(),
		}
	}
}
impl<Message> Default for State<Message> {
	fn default() -> Self {
		Self {
			date: Local::now().naive_local().date(),
			on_submit: None,
			on_close: None,
			focus: Focus::default(),
			keyboard_modifiers: keyboard::Modifiers::default(),
		}
	}
}

/// Just a workaround to pass the button states from the tree to the overlay
#[allow(missing_debug_implementations)]
pub struct DatePickerOverlayButtons<'a, Message, Theme>
where
	Message: Clone,
	Theme: crate::style::Catalog + iced::widget::button::Catalog,
{
	/// The cancel button of the [`DatePickerOverlay`].
	cancel_button: Element<'a, Message, Theme, Renderer>,
	/// The submit button of the [`DatePickerOverlay`].
	submit_button: Element<'a, Message, Theme, Renderer>,
}

impl<'a, Message, Theme> Default for DatePickerOverlayButtons<'a, Message, Theme>
where
	Message: 'a + Clone,
	Theme: 'a
		+ crate::style::Catalog
		+ iced::widget::button::Catalog
		+ iced::widget::text::Catalog
		+ iced::widget::container::Catalog,
{
	fn default() -> Self {
		Self {
			cancel_button: Button::new(
				text::Text::new(icon_to_string(RequiredIcons::X))
					.font(REQUIRED_FONT)
					.align_x(Horizontal::Center)
					.width(Length::Fill),
			)
			.into(),
			submit_button: Button::new(
				text::Text::new(icon_to_string(RequiredIcons::Check))
					.font(REQUIRED_FONT)
					.align_x(Horizontal::Center)
					.width(Length::Fill),
			)
			.into(),
		}
	}
}

#[allow(clippy::unimplemented)]
impl<Message, Theme> Widget<Message, Theme, Renderer>
	for DatePickerOverlayButtons<'_, Message, Theme>
where
	Message: Clone,
	Theme: crate::style::Catalog + iced::widget::button::Catalog + iced::widget::container::Catalog,
{
	fn children(&self) -> Vec<Tree> {
		vec![
			Tree::new(&self.cancel_button),
			Tree::new(&self.submit_button),
		]
	}

	fn diff(&self, tree: &mut Tree) {
		tree.diff_children(&[&self.cancel_button, &self.submit_button]);
	}

	fn size(&self) -> Size<Length> {
		unimplemented!("This should never be reached!")
	}

	fn layout(&self, _tree: &mut Tree, _renderer: &Renderer, _limits: &Limits) -> Node {
		unimplemented!("This should never be reached!")
	}

	fn draw(
		&self,
		_state: &Tree,
		_renderer: &mut Renderer,
		_theme: &Theme,
		_style: &renderer::Style,
		_layout: Layout<'_>,
		_cursor: Cursor,
		_viewport: &Rectangle,
	) {
		unimplemented!("This should never be reached!")
	}
}

impl<'a, Message, Theme> From<DatePickerOverlayButtons<'a, Message, Theme>>
	for Element<'a, Message, Theme, Renderer>
where
	Message: 'a + Clone,
	Theme: 'a
		+ crate::style::Catalog
		+ iced::widget::button::Catalog
		+ iced::widget::container::Catalog,
{
	fn from(overlay: DatePickerOverlayButtons<'a, Message, Theme>) -> Self {
		Self::new(overlay)
	}
}

/// An enumeration of all focusable elements of the [`DatePickerOverlay`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Focus {
	/// Nothing is in focus.
	None,

	/// The overlay itself is in focus.
	Overlay,

	/// The month area is in focus.
	Month,

	/// The year area is in focus.
	Year,

	/// The day area is in focus.
	Day,

	/// The cancel button is in focus.
	Cancel,

	/// The submit button is in focus.
	Submit,
}

impl Focus {
	/// Gets the next focusable element.
	#[must_use]
	pub const fn next(self) -> Self {
		match self {
			Self::Overlay => Self::Month,
			Self::Month => Self::Year,
			Self::Year => Self::Day,
			Self::Day => Self::Cancel,
			Self::Cancel => Self::Submit,
			Self::Submit | Self::None => Self::Overlay,
		}
	}

	/// Gets the previous focusable element.
	#[must_use]
	pub const fn previous(self) -> Self {
		match self {
			Self::None => Self::None,
			Self::Overlay => Self::Submit,
			Self::Month => Self::Overlay,
			Self::Year => Self::Month,
			Self::Day => Self::Year,
			Self::Cancel => Self::Day,
			Self::Submit => Self::Cancel,
		}
	}
}

impl Default for Focus {
	fn default() -> Self {
		Self::None
	}
}

/// Draws the month/year row
fn month_year(
	renderer: &mut Renderer,
	layout: Layout<'_>,
	date: &str,
	cursor: Point,
	//style: &Style,
	style: &HashMap<StyleState, Style>,
	focus: Focus,
	font_size: Pixels,
) {
	let mut children = layout.children();

	let month_layout = children
		.next()
		.expect("Graphics: Layout should have a month layout");
	let year_layout = children
		.next()
		.expect("Graphics: Layout should have a year layout");

	let mut f = |layout: Layout<'_>, text: &str, target: Focus, font_size: Pixels| {
		let style_state = if focus == target {
			StyleState::Focused
		} else {
			StyleState::Active
		};

		let mut children = layout.children();

		let left_bounds = children
			.next()
			.expect("Graphics: Layout should have a left arrow layout")
			.bounds();
		let center_bounds = children
			.next()
			.expect("Graphics: Layout should have a center layout")
			.bounds();
		let right_bounds = children
			.next()
			.expect("Graphics: Layout should have a right arrow layout")
			.bounds();

		let left_arrow_hovered = left_bounds.contains(cursor);
		let right_arrow_hovered = right_bounds.contains(cursor);

		let bounds = layout.bounds();
		if (style_state == StyleState::Focused) && (bounds.width > 0.) && (bounds.height > 0.) {
			renderer.fill_quad(
				renderer::Quad {
					bounds,
					border: Border {
						radius: style
							.get(&style_state)
							.expect("Style Sheet not found.")
							.border_radius
							.into(),
						width: style
							.get(&style_state)
							.expect("Style Sheet not found.")
							.border_width,
						color: style
							.get(&style_state)
							.expect("Style Sheet not found.")
							.border_color,
					},
					shadow: Shadow::default(),
				},
				style
					.get(&style_state)
					.expect("Style Sheet not found.")
					.background,
			);
		}

		// Left caret
		renderer.fill_text(
			iced::advanced::Text {
				content: icon_to_string(RequiredIcons::CaretLeftFill),
				bounds: Size::new(left_bounds.width, left_bounds.height),
				size: Pixels(font_size.0 + if left_arrow_hovered { 1.0 } else { 0.0 }),
				font: REQUIRED_FONT,
				horizontal_alignment: Horizontal::Center,
				vertical_alignment: Vertical::Center,
				line_height: text::LineHeight::Relative(1.3),
				shaping: text::Shaping::Advanced,
				wrapping: Wrapping::default(),
			},
			Point::new(left_bounds.center_x(), left_bounds.center_y()),
			style
				.get(&style_state)
				.expect("Style Sheet not found.")
				.text_color,
			left_bounds,
		);

		// Text
		renderer.fill_text(
			iced::advanced::Text {
				content: text.to_owned(),
				bounds: Size::new(center_bounds.width, center_bounds.height),
				size: font_size,
				font: renderer.default_font(),
				horizontal_alignment: Horizontal::Center,
				vertical_alignment: Vertical::Center,
				line_height: text::LineHeight::Relative(1.3),
				shaping: text::Shaping::Basic,
				wrapping: Wrapping::default(),
			},
			Point::new(center_bounds.center_x(), center_bounds.center_y()),
			style
				.get(&style_state)
				.expect("Style Sheet not found.")
				.text_color,
			center_bounds,
		);

		// Right caret
		renderer.fill_text(
			iced::advanced::Text {
				content: icon_to_string(RequiredIcons::CaretRightFill),
				bounds: Size::new(right_bounds.width, right_bounds.height),
				size: Pixels(font_size.0 + if right_arrow_hovered { 1.0 } else { 0.0 }),
				font: REQUIRED_FONT,
				horizontal_alignment: Horizontal::Center,
				vertical_alignment: Vertical::Center,
				line_height: text::LineHeight::Relative(1.3),
				shaping: text::Shaping::Advanced,
				wrapping: Wrapping::default(),
			},
			Point::new(right_bounds.center_x(), right_bounds.center_y()),
			style
				.get(&style_state)
				.expect("Style Sheet not found.")
				.text_color,
			right_bounds,
		);
	};

	let (year, month) = date.split_once(' ').expect("Date must contain space");

	// Draw month
	f(month_layout, month, Focus::Month, font_size);

	// Draw year
	f(year_layout, year, Focus::Year, font_size);
}

/// Draws the days
fn days(
	renderer: &mut Renderer,
	layout: Layout<'_>,
	date: chrono::NaiveDate,
	cursor: Point,
	//style: &Style,
	style: &HashMap<StyleState, Style>,
	focus: Focus,
	font_size: Pixels,
) {
	let mut children = layout.children();

	let day_labels_layout = children
		.next()
		.expect("Graphics: Layout should have a day labels layout");
	day_labels(renderer, day_labels_layout, style, focus, font_size);

	day_table(
		renderer,
		&mut children,
		date,
		cursor,
		style,
		focus,
		font_size,
	);
}

/// Draws the day labels
fn day_labels(
	renderer: &mut Renderer,
	layout: Layout<'_>,
	style: &HashMap<StyleState, Style>,
	_focus: Focus,
	font_size: Pixels,
) {
	for (i, label) in layout.children().enumerate() {
		let bounds = label.bounds();

		renderer.fill_text(
			iced::advanced::Text {
				content: crate::date::WEEKDAY_LABELS[i].clone(),
				bounds: Size::new(bounds.width, bounds.height),
				size: font_size,
				font: renderer.default_font(),
				horizontal_alignment: Horizontal::Center,
				vertical_alignment: Vertical::Center,
				line_height: text::LineHeight::Relative(1.3),
				shaping: text::Shaping::Basic,
				wrapping: Wrapping::default(),
			},
			Point::new(bounds.center_x(), bounds.center_y()),
			style
				.get(&StyleState::Active)
				.expect("Style Sheet not found.")
				.text_color,
			bounds,
		);
	}
}

/// Draws the day table
fn day_table(
	renderer: &mut Renderer,
	children: &mut dyn Iterator<Item = Layout<'_>>,
	date: chrono::NaiveDate,
	cursor: Point,
	style: &HashMap<StyleState, Style>,
	focus: Focus,
	font_size: Pixels,
) {
	for (y, row) in children.enumerate() {
		for (x, label) in row.children().enumerate() {
			let bounds = label.bounds();
			let (number, is_in_month) =
				crate::date::position_to_day(x, y, date.year(), date.month());

			let mouse_over = bounds.contains(cursor);

			let selected = date.day() == number as u32 && is_in_month == IsInMonth::Same;

			let mut style_state = StyleState::Active;
			if selected {
				style_state = style_state.max(StyleState::Selected);
			}
			if mouse_over {
				style_state = style_state.max(StyleState::Hovered);
			}

			if (bounds.width > 0.) && (bounds.height > 0.) {
				renderer.fill_quad(
					renderer::Quad {
						bounds,
						border: Border {
							radius: (bounds.height / 2.0).into(),
							width: 0.0,
							color: Color::TRANSPARENT,
						},
						shadow: Shadow::default(),
					},
					style
						.get(&style_state)
						.expect("Style Sheet not found.")
						.day_background,
				);

				if focus == Focus::Day && selected {
					renderer.fill_quad(
						renderer::Quad {
							bounds,
							border: Border {
								radius: style
									.get(&StyleState::Focused)
									.expect("Style Sheet not found.")
									.border_radius
									.into(),
								width: style
									.get(&StyleState::Focused)
									.expect("Style Sheet not found.")
									.border_width,
								color: style
									.get(&StyleState::Focused)
									.expect("Style Sheet not found.")
									.border_color,
							},
							shadow: Shadow::default(),
						},
						Color::TRANSPARENT,
					);
				}
			}

			renderer.fill_text(
				iced::advanced::Text {
					content: format!("{number:02}"), // Todo: is there some way of static format as this has a fixed size?
					bounds: Size::new(bounds.width, bounds.height),
					size: font_size,
					font: renderer.default_font(),
					horizontal_alignment: Horizontal::Center,
					vertical_alignment: Vertical::Center,
					line_height: text::LineHeight::Relative(1.3),
					shaping: text::Shaping::Basic,
					wrapping: Wrapping::default(),
				},
				Point::new(bounds.center_x(), bounds.center_y()),
				if is_in_month == IsInMonth::Same {
					style
						.get(&style_state)
						.expect("Style Sheet not found.")
						.text_color
				} else {
					style
						.get(&style_state)
						.expect("Style Sheet not found.")
						.text_attenuated_color
				},
				bounds,
			);
		}
	}
}
