use iced::{theme, widget::{button, column, row, scrollable, text, Column}, Element, Length, Padding};
use crate::{components::{colored_horizontal_seperator, horizontal_seperator, loading_screen}, core::{OrderedHashMap, Project, ProjectId}, project_tracker::{ProjectTrackerApp, UiMessage}, styles::{scrollable_vertical_direction, ProjectPreviewButtonStyle, ScrollableStyle, LARGE_TEXT_SIZE, PADDING_AMOUNT, SCROLLBAR_WIDTH, SMALL_PADDING_AMOUNT, SMALL_SPACING_AMOUNT, SPACING_AMOUNT, TITLE_TEXT_SIZE}};

#[derive(Clone)]
pub struct OverviewPage {

}

impl Default for OverviewPage {
	fn default() -> Self {
		Self::new()
	}
}

impl OverviewPage {
	pub fn new() -> Self {
		Self {

		}
	}

	fn todo_tasks_list(projects: &OrderedHashMap<ProjectId, Project>) -> Element<UiMessage> {
		scrollable(
			Column::from_vec(projects.iter()
				.filter(|(_project_id, project)| {
					project.tasks.values()
						.filter(|t| t.is_todo())
						.count() != 0
				})
				.map(|(project_id, project)| {
					let task_list = project.tasks.iter()
						.filter(|(_, task)| task.is_todo())
						.map(|(_, task)| {
							row![
								text("-"),
								text(&task.name)
							]
							.spacing(SMALL_SPACING_AMOUNT)
							.into()
						})
						.collect();

					button(column![
						text(&project.name).size(LARGE_TEXT_SIZE),

						colored_horizontal_seperator(project.color.into()),

						Column::from_vec(task_list)
							.padding(Padding{ left: PADDING_AMOUNT, ..Padding::ZERO }),
					])
					.width(Length::Fill)
					.style(theme::Button::custom(ProjectPreviewButtonStyle{ selected: false, color: None }))
					.on_press(UiMessage::SelectProject(Some(project_id)))
					.into()
				})
				.collect()
			)
			.width(Length::Fill)
			.spacing(SMALL_SPACING_AMOUNT + SPACING_AMOUNT)
			.padding(Padding{ right: SCROLLBAR_WIDTH + SMALL_PADDING_AMOUNT, ..Padding::ZERO })
		)
		.style(theme::Scrollable::custom(ScrollableStyle))
		.direction(scrollable_vertical_direction())
		.into()
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<UiMessage> {
		if let Some(database) = &app.database {
			column![
				text("Overview").size(TITLE_TEXT_SIZE),

				horizontal_seperator(),

				Self::todo_tasks_list(&database.projects),
			]
			.width(Length::Fill)
			.spacing(SPACING_AMOUNT)
			.padding(Padding{ left: PADDING_AMOUNT, right: 0.0, top: PADDING_AMOUNT, bottom: PADDING_AMOUNT })
			.into()
		}
		else {
			loading_screen()
		}
	}
}
