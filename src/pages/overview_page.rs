use std::collections::HashMap;

use iced::{theme, widget::{button, column, row, scrollable, text, Column}, Element, Length, Padding};
use crate::{components::{horizontal_seperator, loading_screen}, project::{Project, ProjectId, TaskFilter}, project_tracker::{ProjectTrackerApp, UiMessage}, styles::{ProjectPreviewButtonStyle, HORIZONTAL_PADDING, LARGE_TEXT_SIZE, PADDING_AMOUNT, SMALL_SPACING_AMOUNT, SPACING_AMOUNT, TITLE_TEXT_SIZE}};

#[derive(Debug, Clone)]
pub struct OverviewPage {

}

impl OverviewPage {
	pub fn new() -> Self {
		Self {

		}
	}

	fn todo_tasks_list(projects: &HashMap<ProjectId, Project>) -> Element<UiMessage> {
		scrollable(
			Column::from_vec(projects.values()
				.filter(|p| {
					p.tasks.values()
					.filter(|t| !t.is_done())
					.count() != 0
				})
				.map(|project| {
					let task_list = project.tasks.values()
						.filter(|t| TaskFilter::Todo.matches(t))
						.map(|t| {
							row![
								text("-"),
								text(&t.name)
							]
							.spacing(SMALL_SPACING_AMOUNT)
							.into()
						})
						.collect();

					button(column![
						text(&project.name).size(LARGE_TEXT_SIZE),
						horizontal_seperator(),
						Column::from_vec(task_list).padding(Padding{ left: PADDING_AMOUNT, ..Padding::ZERO }),
					])
					.width(Length::Fill)
					.style(theme::Button::custom(ProjectPreviewButtonStyle{ selected: false }))
					.on_press(UiMessage::SelectProject(project.id))
					.into()
				})
				.collect()
			)
			.width(Length::Fill)
			.spacing(SMALL_SPACING_AMOUNT + SPACING_AMOUNT)
		)
		.into()
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<UiMessage> {
		if let Some(saved_state) = &app.saved_state {
			column![
				text("Overview").size(TITLE_TEXT_SIZE),
			
				horizontal_seperator(),

				Self::todo_tasks_list(&saved_state.projects),
			]
			.width(Length::Fill)
			.spacing(SPACING_AMOUNT)
			.padding(HORIZONTAL_PADDING)
			.into()
		}
		else {
			loading_screen()
		}
	}
}