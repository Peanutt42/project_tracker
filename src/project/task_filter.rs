use iced::{Element, widget::{button, text, row}, alignment::Vertical, Length, Padding, theme};
use crate::{project::Task, project_tracker::UiMessage, pages::ProjectPageMessage, styles::{GreenButtonStyle, SPACING_AMOUNT}};

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
			button(text(label).vertical_alignment(Vertical::Center))
				.style(
					if filter == current_filter {
						theme::Button::custom(GreenButtonStyle)
					}
					else {
						theme::Button::Text
					}
				)
				.padding(Padding{ left: 5.0, right: 5.0, top: 2.5, bottom: 2.5 })
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