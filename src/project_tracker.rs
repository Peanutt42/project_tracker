use iced::{widget::{button, column, scrollable, text}, Element, Length, Sandbox, Theme};
use iced_aw::{Grid, GridRow, modal};
use crate::{components::{home_button, project_preview, CreateNewProjectModal}, project::Project};
use crate::task::{Task, TaskState};

#[derive(Debug, Clone)]
pub enum ProjectTrackerPage {
	StartPage {
		create_new_project_modal: CreateNewProjectModal,
	},
	ProjectPage {
		project_name: String,
	},
}

pub struct ProjectTrackerApp {
	pub page: ProjectTrackerPage,
	pub projects: Vec<Project>,
	pub dark_mode: bool,
}

#[derive(Debug, Clone)]
pub enum UiMessage {
	SwitchPage(ProjectTrackerPage),
	ToggleTheme,
	OpenCreateNewProjectModal,
	CloseCreateNewProjectModal,
	ChangeCreateNewProjectName(String),
	CreateProject(String),
}


impl Sandbox for ProjectTrackerApp {
	type Message = UiMessage;

	fn new() -> Self {
		let mut projects = Vec::new();
		for i in 0..2 {
			projects.push(Project::new(format!("Project Tracker {i}"), vec![Task::new("Project Page".to_string(), TaskState::Todo)]));
			projects.push(Project::new(format!("Client Server {i}"), vec![Task::new("Packet Loss and Latency Simulator".to_string(), TaskState::Done)]));
			projects.push(Project::new(format!("SphynxEngine {i}"), vec![]));
			projects.push(Project::new(format!("SIA Project {i}"), vec![
				Task::new("PID-Konstanten optimieren".to_string(), TaskState::InProgress),
				Task::new("Finale Strecke bauen".to_string(), TaskState::Todo),
				Task::new("Ausweichspur".to_string(), TaskState::Done),
			]));
		}

		Self {
			page: ProjectTrackerPage::StartPage{
				create_new_project_modal: CreateNewProjectModal::new(),
			},
			projects,
			dark_mode: true,
		}
	}

	fn title(&self) -> String {
		"Project Tracker".to_string()
	}

	fn theme(&self) -> Theme {
		if self.dark_mode {
			Theme::Dark
		}
		else {
			Theme::Light
		}
	}

	fn update(&mut self, message: UiMessage) {
		match message {
			UiMessage::SwitchPage(new_page) => self.page = new_page,
			UiMessage::ToggleTheme => self.dark_mode = !self.dark_mode,
			UiMessage::OpenCreateNewProjectModal => {
				if let ProjectTrackerPage::StartPage { create_new_project_modal } = &mut self.page {
					create_new_project_modal.open();
				}
			},
			UiMessage::CloseCreateNewProjectModal => {
				if let ProjectTrackerPage::StartPage { create_new_project_modal } = &mut self.page {
					create_new_project_modal.close();
				}
			},
			UiMessage::ChangeCreateNewProjectName(new_project_name) => {
				if let ProjectTrackerPage::StartPage { create_new_project_modal } = &mut self.page {
					create_new_project_modal.project_name = new_project_name;
				}
			},
			UiMessage::CreateProject(project_name) => {
				self.projects.push(Project::new(project_name, Vec::new()));
				if let ProjectTrackerPage::StartPage { create_new_project_modal } = &mut self.page {
					create_new_project_modal.close();
				}
			},
		}
	}

	fn view(&self) -> Element<UiMessage> {
		match &self.page {
			ProjectTrackerPage::StartPage { create_new_project_modal } => {
				let project_rows: Vec<GridRow<UiMessage>> = self.projects
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

				let create_new_project_button = button("Create New").on_press(UiMessage::OpenCreateNewProjectModal);

				let background = column![
					create_new_project_button,
					project_grid,
				];

				modal(background, create_new_project_modal.view())
					.backdrop(UiMessage::CloseCreateNewProjectModal)
					.on_esc(UiMessage::CloseCreateNewProjectModal)
					.into()
			},
			ProjectTrackerPage::ProjectPage { project_name } => {
				let mut current_project = None;
				for project in self.projects.iter() {
					if project.name == *project_name {
						current_project = Some(project);
						break;
					}
				}
				let project_element = if let Some(project) = current_project {
					project.view()
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
