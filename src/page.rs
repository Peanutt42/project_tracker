use iced::{widget::{button, column, scrollable, text}, Element, Length};
use iced_aw::{Grid, GridRow, modal};
use crate::{project::Project, project_tracker::UiMessage};
use crate::components::{home_button, project_preview, CreateNewProjectModal, create_new_project_button, CreateNewTaskModal};



#[derive(Debug, Clone)]
pub enum Page {
	StartPage {
		create_new_project_modal: CreateNewProjectModal,
	},
	ProjectPage {
		project_name: String,
		create_new_task_modal: CreateNewTaskModal,
	},
}

impl Page {
	pub fn view<'a>(&'a self, projects: &'a [Project]) -> Element<UiMessage> {
		match self {
			Page::StartPage { create_new_project_modal } => {
				let project_rows: Vec<GridRow<UiMessage>> = projects
					.chunks(4)
					.map(|project_chunk| {
					let project_row_views: Vec<Element<UiMessage>> = project_chunk
						.iter()
						.map(|project| {
							project_preview(project).into()
						})
						.collect();
					GridRow::with_elements(project_row_views)
				}).collect();

				let project_grid = scrollable(
					Grid::with_rows(project_rows)
						.width(Length::Fill)
						.spacing(10.0)
						.padding(10)
				)
				.width(Length::Fill);

				let create_new_project_button = create_new_project_button();

				let background = column![
					button(text("Save"))
						.on_press(UiMessage::Save),
					create_new_project_button,
					project_grid,
				];

				modal(background, create_new_project_modal.view())
					.backdrop(UiMessage::CloseCreateNewProjectModal)
					.on_esc(UiMessage::CloseCreateNewProjectModal)
					.into()
			},
			Page::ProjectPage { project_name, create_new_task_modal } => {
				let mut current_project = None;
				for project in projects.iter() {
					if project.name == *project_name {
						current_project = Some(project);
						break;
					}
				}
				let project_element = if let Some(project) = current_project {
					project.view(create_new_task_modal)
				}
				else {
					text("Invalid Project").into()
				};
				column![
					home_button(),
					project_element,
					button("theme").on_press(UiMessage::ToggleTheme)
				].into()
			}
		}
	}
}
