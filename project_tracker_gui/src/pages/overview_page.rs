use iced::{theme, widget::{button, column, row, text, Column}, Element, Length, Padding};
use crate::{components::{vertical_scrollable, horizontal_seperator_colored, horizontal_seperator, loading_screen}, core::{OrderedHashMap, Project, ProjectId}, project_tracker::{ProjectTrackerApp, UiMessage}, styles::{ProjectPreviewButtonStyle, LARGE_TEXT_SIZE, PADDING_AMOUNT, SMALL_SPACING_AMOUNT, SPACING_AMOUNT, TITLE_TEXT_SIZE}};

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

	fn todo_tasks_list(project: &Project) -> Element<UiMessage> {
		let task_list = project.todo_tasks.iter()
			.map(|(_task_id, task)| {
				row![
					text("\u{2022}"), // bullet-point:'•'
					text(&task.name)
				]
				.spacing(SMALL_SPACING_AMOUNT)
				.into()
			})
			.collect();

		Column::from_vec(task_list)
			.padding(Padding{ left: PADDING_AMOUNT, ..Padding::ZERO })
			.into()
	}

	fn todo_tasks_lists(projects: &OrderedHashMap<ProjectId, Project>) -> Element<UiMessage> {
		vertical_scrollable(
			Column::from_vec(projects.iter()
				.filter(|(_project_id, project)|
					!project.todo_tasks.is_empty()
				)
				.map(|(project_id, project)| {

					button(column![
						text(&project.name).size(LARGE_TEXT_SIZE),

						horizontal_seperator_colored(project.color.into()),

						Self::todo_tasks_list(project),
					])
					.width(Length::Fill)
					.style(theme::Button::custom(ProjectPreviewButtonStyle{ selected: false, project_color: Some(project.color.into()) }))
					.on_press(UiMessage::SelectProject(Some(project_id)))
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
		if let Some(database) = &app.database {
			column![
				text("Overview").size(TITLE_TEXT_SIZE),

				horizontal_seperator(),

				Self::todo_tasks_lists(database.projects()),
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
