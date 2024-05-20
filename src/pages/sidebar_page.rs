use iced::{alignment::{Alignment, Horizontal}, widget::{column, container, row, scrollable, text_input, Column}, Command, Element, Length};
use crate::{components::{create_new_project_button, cancel_button, loading_screen, overview_button, partial_horizontal_seperator, project_preview, settings_button}, project_tracker::UiMessage};
use crate::styles::{HORIZONTAL_PADDING, SPACING_AMOUNT, LARGE_SPACING_AMOUNT};
use crate::project_tracker::ProjectTrackerApp;
use crate::project::Project;

#[derive(Debug, Clone)]
pub enum SidebarPageMessage {
	OpenCreateNewProject,
	CloseCreateNewProject,
	ChangeCreateNewProjectName(String),
}

impl From<SidebarPageMessage> for UiMessage {
	fn from(value: SidebarPageMessage) -> Self {
		UiMessage::SidebarPageMessage(value)
	}
}

#[derive(Debug, Clone)]
pub struct SidebarPage {
	create_new_project_name: Option<String>,
}

impl SidebarPage {
	pub fn new() -> Self {
		Self {
			create_new_project_name: None,
		}
	}

	fn project_preview_list<'a>(projects: &'a [Project], app: &'a ProjectTrackerApp, overview_button: Column<'a, UiMessage>) -> Element<'a, UiMessage> {
		let mut list: Vec<Element<UiMessage>> = projects.iter()
		.map(|project| {
			let selected = project.name == app.selected_page_name;
			project_preview(project, selected)
		})
		.collect();
		list.insert(0, overview_button.into());

		scrollable(
			Column::from_vec(list)
				.width(Length::Fill)
				.spacing(SPACING_AMOUNT)
		)
		.width(Length::Fill)
		.height(Length::Shrink)
		.into()
	}

	pub fn update(&mut self, message: SidebarPageMessage) -> Command<UiMessage> {
		match message {
			SidebarPageMessage::OpenCreateNewProject => { self.create_new_project_name = Some(String::new()); Command::none() },
			SidebarPageMessage::CloseCreateNewProject => { self.create_new_project_name = None; Command::none() },
			SidebarPageMessage::ChangeCreateNewProjectName(new_project_name) => { self.create_new_project_name = Some(new_project_name); Command::none() },
		}		
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<UiMessage> {
		let overview_button = column![
			overview_button(app.content_page.is_overview_page()),
			partial_horizontal_seperator(2.5),
		]
		.spacing(LARGE_SPACING_AMOUNT);

		let list: Element<UiMessage> =
			if let Some(saved_state) = &app.saved_state {
				Self::project_preview_list(&saved_state.projects, app, overview_button)
			}
			else {
				column![
					overview_button,
					loading_screen(),
				]
				.width(Length::Fill)
				.spacing(SPACING_AMOUNT)
				.into()
			};

		let create_new_project_element: Element<UiMessage> = if let Some(create_new_project_name) = &self.create_new_project_name {
			let new_project_name = self.create_new_project_name.clone().unwrap_or(String::from("<no project name input>"));
			
			row![
				text_input("New project name", create_new_project_name)
					.on_input(|input| SidebarPageMessage::ChangeCreateNewProjectName(input).into())
					.on_submit(UiMessage::CreateProject(new_project_name)),

				cancel_button()
					.on_press(SidebarPageMessage::CloseCreateNewProject.into())
			]
			.align_items(Alignment::Center)
			.into()
		}
		else {
			container(create_new_project_button())
				.align_x(Horizontal::Center)
				.width(Length::Fill)
				.into()
		};

		column![
			column![
				list,
	
				column![
					partial_horizontal_seperator(2.5),
	
					create_new_project_element,
				]
				.spacing(LARGE_SPACING_AMOUNT)
			]
			.spacing(SPACING_AMOUNT)
			.padding(HORIZONTAL_PADDING),
			
			container(settings_button())
				.height(Length::Fill)
				.align_y(iced::alignment::Vertical::Bottom)
		]
		.into()
	}
}