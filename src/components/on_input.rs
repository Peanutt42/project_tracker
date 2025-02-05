use iced::{
	advanced::Widget,
	keyboard::{self, key::Named, Key},
	mouse::{self, Button},
	Element, Event, Renderer, Theme,
};

pub struct OnInput<'a, Message: 'a + Clone> {
	content: Element<'a, Message>,
	on_esc: Option<Message>,
	on_mouse_forward: Option<Message>,
	on_mouse_backward: Option<Message>,
}

impl<'a, Message: 'a + Clone> OnInput<'a, Message> {
	pub fn new(content: Element<'a, Message>) -> Self {
		Self {
			content,
			on_esc: None,
			on_mouse_forward: None,
			on_mouse_backward: None,
		}
	}

	pub fn on_esc(self, on_esc: Message) -> Self {
		Self {
			on_esc: Some(on_esc),
			..self
		}
	}
	pub fn on_mouse_forward(self, on_mouse_forward: Message) -> Self {
		Self {
			on_mouse_forward: Some(on_mouse_forward),
			..self
		}
	}
	pub fn on_mouse_backward(self, on_mouse_backward: Message) -> Self {
		Self {
			on_mouse_backward: Some(on_mouse_backward),
			..self
		}
	}
}

impl<'a, Message: 'a + Clone> Widget<Message, Theme, Renderer> for OnInput<'a, Message> {
	fn tag(&self) -> iced::advanced::widget::tree::Tag {
		self.content.as_widget().tag()
	}

	fn state(&self) -> iced::advanced::widget::tree::State {
		self.content.as_widget().state()
	}

	fn children(&self) -> Vec<iced::advanced::widget::Tree> {
		self.content.as_widget().children()
	}

	fn diff(&self, tree: &mut iced::advanced::widget::Tree) {
		self.content.as_widget().diff(tree)
	}

	fn size(&self) -> iced::Size<iced::Length> {
		self.content.as_widget().size()
	}

	fn layout(
		&self,
		tree: &mut iced::advanced::widget::Tree,
		renderer: &Renderer,
		limits: &iced::advanced::layout::Limits,
	) -> iced::advanced::layout::Node {
		self.content.as_widget().layout(tree, renderer, limits)
	}

	fn operate(
		&self,
		state: &mut iced::advanced::widget::Tree,
		layout: iced::advanced::Layout<'_>,
		renderer: &Renderer,
		operation: &mut dyn iced::advanced::widget::Operation,
	) {
		operation.container(None, layout.bounds(), &mut |operation| {
			self.content
				.as_widget()
				.operate(state, layout, renderer, operation)
		})
	}

	fn on_event(
		&mut self,
		state: &mut iced::advanced::widget::Tree,
		event: iced::Event,
		layout: iced::advanced::Layout<'_>,
		cursor: iced::advanced::mouse::Cursor,
		renderer: &Renderer,
		clipboard: &mut dyn iced::advanced::Clipboard,
		shell: &mut iced::advanced::Shell<'_, Message>,
		viewport: &iced::Rectangle,
	) -> iced::advanced::graphics::core::event::Status {
		let cursor_hovering = cursor.is_over(layout.bounds());

		match &event {
			Event::Mouse(mouse::Event::ButtonPressed(Button::Forward)) if cursor_hovering => {
				match &self.on_mouse_forward {
					Some(on_mouse_forward) => {
						shell.publish(on_mouse_forward.clone());
						iced::advanced::graphics::core::event::Status::Captured
					}
					None => self.content.as_widget_mut().on_event(
						state, event, layout, cursor, renderer, clipboard, shell, viewport,
					),
				}
			}
			Event::Mouse(mouse::Event::ButtonPressed(Button::Back)) if cursor_hovering => {
				match &self.on_mouse_backward {
					Some(on_mouse_backward) => {
						shell.publish(on_mouse_backward.clone());
						iced::advanced::graphics::core::event::Status::Captured
					}
					None => self.content.as_widget_mut().on_event(
						state, event, layout, cursor, renderer, clipboard, shell, viewport,
					),
				}
			}
			Event::Keyboard(keyboard::Event::KeyPressed {
				key: Key::Named(Named::Escape),
				..
			}) => match &self.on_esc {
				Some(on_esc) => {
					shell.publish(on_esc.clone());
					iced::advanced::graphics::core::event::Status::Captured
				}
				None => self.content.as_widget_mut().on_event(
					state, event, layout, cursor, renderer, clipboard, shell, viewport,
				),
			},
			_ => self.content.as_widget_mut().on_event(
				state, event, layout, cursor, renderer, clipboard, shell, viewport,
			),
		}
	}

	fn mouse_interaction(
		&self,
		state: &iced::advanced::widget::Tree,
		layout: iced::advanced::Layout<'_>,
		cursor: iced::advanced::mouse::Cursor,
		viewport: &iced::Rectangle,
		renderer: &Renderer,
	) -> iced::advanced::mouse::Interaction {
		self.content
			.as_widget()
			.mouse_interaction(state, layout, cursor, viewport, renderer)
	}

	fn draw(
		&self,
		tree: &iced::advanced::widget::Tree,
		renderer: &mut Renderer,
		theme: &Theme,
		style: &iced::advanced::renderer::Style,
		layout: iced::advanced::Layout<'_>,
		cursor: iced::advanced::mouse::Cursor,
		viewport: &iced::Rectangle,
	) {
		self.content
			.as_widget()
			.draw(tree, renderer, theme, style, layout, cursor, viewport)
	}

	fn overlay<'b>(
		&'b mut self,
		state: &'b mut iced::advanced::widget::Tree,
		layout: iced::advanced::Layout<'_>,
		renderer: &Renderer,
		translation: iced::Vector,
	) -> Option<iced::advanced::overlay::Element<'b, Message, Theme, Renderer>> {
		self.content
			.as_widget_mut()
			.overlay(state, layout, renderer, translation)
	}
}

impl<'a, Message: 'a + Clone> From<OnInput<'a, Message>> for Element<'a, Message> {
	fn from(value: OnInput<'a, Message>) -> Self {
		Self::new(value)
	}
}

pub fn on_input<'a, Message: 'a + Clone>(
	content: impl Into<Element<'a, Message>>,
) -> OnInput<'a, Message> {
	OnInput::new(content.into())
}
