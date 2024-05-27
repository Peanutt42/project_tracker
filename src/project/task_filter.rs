use iced::{theme, widget::{button, row, text}, Element, Length, Padding};
use crate::{pages::ProjectPageMessage, project::Task, project_tracker::UiMessage, styles::{TaskFilterButtonStyle, PADDING_AMOUNT, SPACING_AMOUNT}};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TaskFilter {
	#[default]
	All,
	Todo,
	Done,
}

impl TaskFilter {
	pub fn matches(self, task: &Task) -> bool {
		match self {
			TaskFilter::All => true,
			TaskFilter::Todo => !task.is_done(),
			TaskFilter::Done => task.is_done(),
		}
	}

	pub fn view(&self) -> Element<UiMessage> {
		let filter_button = |label, filter, current_filter| {
			button(text(label))
				.style(theme::Button::custom(TaskFilterButtonStyle{ selected: filter == current_filter }))
				.padding(Padding{ left: PADDING_AMOUNT, right: PADDING_AMOUNT, top: 3.5, bottom: 3.5 })
				.on_press(ProjectPageMessage::ChangeTaskFilter(filter).into())
		};

		row![
			filter_button("All", TaskFilter::All, *self),
			filter_button("Todo", TaskFilter::Todo, *self),
			filter_button("Done", TaskFilter::Done, *self)
		]
		.width(Length::Shrink)
		.spacing(SPACING_AMOUNT)
		.into()
	}
}