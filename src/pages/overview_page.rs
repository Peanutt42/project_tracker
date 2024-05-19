use iced::{alignment::Horizontal, theme, widget::{button, column, row, scrollable, text, Column}, Alignment, Element, Length, Padding};
use crate::{components::{horizontal_seperator, loading_screen, vertical_seperator}, project::{Project, TaskState}, project_tracker::{ProjectTrackerApp, UiMessage}, styles::ProjectPreviewButtonStyle};

#[derive(Debug, Clone)]
pub struct OverviewPage {

}

impl OverviewPage {
	pub fn new() -> Self {
		Self {

		}
	}

	fn tasks_filtered_by_state_list(projects: &[Project], task_state: TaskState) -> Element<UiMessage> {
		scrollable(
			Column::from_vec(projects.iter()
				.filter(|p| {
					p.tasks.iter()
					.filter(|t| t.state == task_state)
					.count() != 0
				})
				.map(|project| {
					let task_list = project.tasks.iter()
						.filter(|t| t.state == task_state)
						.map(|t| text(&t.name).into())
						.collect();

					button(column![
						text(&project.name).size(20),
						horizontal_seperator(1.0),
						Column::from_vec(task_list).padding(Padding{ left: 10.0, ..Padding::ZERO }),
					])
					.width(Length::Fill)
					.style(theme::Button::custom(ProjectPreviewButtonStyle{ selected: false }))
					.on_press(UiMessage::SelectProject(project.name.clone()))
					.into()
				})
				.collect()
			)
			.width(Length::Fill)
			.spacing(15)
		)
		.into()
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<UiMessage> {
		if let Some(saved_state) = &app.saved_state {
			column![
				text("Todo Overview").size(35),
				row![
					column![
						text("Todo")
							.size(25)
							.width(Length::Fill)
							.horizontal_alignment(Horizontal::Center),
						
						horizontal_seperator(1.0),

						Self::tasks_filtered_by_state_list(&saved_state.projects, TaskState::Todo),
					]
					.width(Length::FillPortion(1)),

					vertical_seperator(1.0),
					
					column![
						text("In Progress")
							.size(25)
							.width(Length::Fill)
							.horizontal_alignment(Horizontal::Center),

						horizontal_seperator(1.0),
						
						Self::tasks_filtered_by_state_list(&saved_state.projects, TaskState::InProgress),
					]
					.width(Length::FillPortion(1)),
				]
				.width(Length::Fill)
			]
			.spacing(10)
			.align_items(Alignment::Center)
			.into()
		}
		else {
			loading_screen()
		}
	}
}