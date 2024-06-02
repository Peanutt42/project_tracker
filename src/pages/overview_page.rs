use iced::{theme, widget::{button, column, row, scrollable, text, Column}, Element, Length, Padding};
use crate::{components::{horizontal_seperator, loading_screen}, core::{OrderedHashMap, Project, ProjectId, TaskFilter}, project_tracker::{ProjectTrackerApp, UiMessage}, styles::{ProjectPreviewButtonStyle, HORIZONTAL_PADDING, LARGE_TEXT_SIZE, PADDING_AMOUNT, SMALL_SPACING_AMOUNT, SPACING_AMOUNT, TITLE_TEXT_SIZE}};

#[derive(Debug, Clone)]
pub struct OverviewPage {

}

impl OverviewPage {
	pub fn new() -> Self {
		Self {

		}
	}

	fn todo_tasks_list(projects: &OrderedHashMap<ProjectId, Project>) -> Element<UiMessage> {
		scrollable(
			Column::from_vec(projects.iter()
				.map(|project_id| {
					(project_id, projects.get(project_id).unwrap())
				})
				.filter(|(_project_id, project)| {
					project.tasks.values()
						.filter(|t| !t.is_done())
						.count() != 0
				})
				.map(|(project_id, project)| {
					let task_list = project.tasks.iter()
						.filter(|task_id| TaskFilter::Todo.matches(project.tasks.get(task_id).unwrap()))
						.map(|task_id| {
							let task = project.tasks.get(task_id).unwrap();

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
						horizontal_seperator(),
						Column::from_vec(task_list).padding(Padding{ left: PADDING_AMOUNT, ..Padding::ZERO }),
					])
					.width(Length::Fill)
					.style(theme::Button::custom(ProjectPreviewButtonStyle{ selected: false }))
					.on_press(UiMessage::SelectProject(*project_id))
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

				Self::todo_tasks_list(&database.projects),
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