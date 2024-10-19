use iced::{
	advanced::Widget,
	keyboard::{self, key::Named, Key},
	Element, Event, Renderer, Theme,
};

pub struct Unfocusable<'a, Message: 'a + Clone> {
	content: Element<'a, Message>,
	on_esc: Message,
}

impl<'a, Message: 'a + Clone> Unfocusable<'a, Message> {
	pub fn new(content: Element<'a, Message>, on_esc: Message) -> Self {
		Self { content, on_esc }
	}
}

impl<'a, Message: 'a + Clone> Widget<Message, Theme, Renderer> for Unfocusable<'a, Message> {
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
		match &event {
			Event::Keyboard(keyboard::Event::KeyPressed {
				key: Key::Named(Named::Escape),
				..
			}) => {
				shell.publish(self.on_esc.clone());
				iced::advanced::graphics::core::event::Status::Captured
			}
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

impl<'a, Message: 'a + Clone> From<Unfocusable<'a, Message>> for Element<'a, Message> {
	fn from(value: Unfocusable<'a, Message>) -> Self {
		Self::new(value)
	}
}

pub fn unfocusable<'a, Message: 'a + Clone>(
	content: impl Into<Element<'a, Message>>,
	on_esc: Message,
) -> Unfocusable<'a, Message> {
	Unfocusable::new(content.into(), on_esc)
}
