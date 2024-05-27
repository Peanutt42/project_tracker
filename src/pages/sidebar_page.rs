use iced::{alignment::{Alignment, Horizontal, Vertical}, theme, widget::{column, container, row, scrollable, text_input, Column}, Command, Element, Length, Padding};
use once_cell::sync::Lazy;
use crate::{components::{cancel_button, create_new_project_button, loading_screen, overview_button, partial_horizontal_seperator, project_preview, settings_button}, project_tracker::UiMessage, styles::TextInputStyle};
use crate::styles::{HORIZONTAL_PADDING, SMALL_PADDING_AMOUNT, SPACING_AMOUNT};
use crate::project_tracker::ProjectTrackerApp;
use crate::project::Project;

static TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

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

	fn project_preview_list<'a>(projects: &'a [Project], app: &'a ProjectTrackerApp) -> Element<'a, UiMessage> {
		let list: Vec<Element<UiMessage>> = projects.iter()
			.map(|project| {
				let selected = project.name == app.selected_page_name;
				project_preview(project, selected)
			})
			.collect();
		
		scrollable(
			Column::from_vec(list)
				.width(Length::Fill)
				.spacing(SPACING_AMOUNT)
		)
		.width(Length::Fill)
		.height(Length::Fill)
		.into()
	}

	pub fn update(&mut self, message: SidebarPageMessage) -> Command<UiMessage> {
		match message {
			SidebarPageMessage::OpenCreateNewProject => { self.create_new_project_name = Some(String::new()); text_input::focus(TEXT_INPUT_ID.clone()) },
			SidebarPageMessage::CloseCreateNewProject => { self.create_new_project_name = None; Command::none() },
			SidebarPageMessage::ChangeCreateNewProjectName(new_project_name) => { self.create_new_project_name = Some(new_project_name); Command::none() },
		}		
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<UiMessage> {
		let list: Element<UiMessage> =
			if let Some(saved_state) = &app.saved_state {
				Self::project_preview_list(&saved_state.projects, app)
			}
			else {
				loading_screen()
			};

		let create_new_project_element: Element<UiMessage> = if let Some(create_new_project_name) = &self.create_new_project_name {
			let new_project_name = self.create_new_project_name.clone().unwrap_or(String::from("<no project name input>"));
			
			container(
				container(
					row![
						text_input("New project name", create_new_project_name)
							.id(TEXT_INPUT_ID.clone())
							.on_input(|input| SidebarPageMessage::ChangeCreateNewProjectName(input).into())
							.on_submit(UiMessage::CreateProject(new_project_name))
							.style(theme::TextInput::Custom(Box::new(TextInputStyle))),
		
						cancel_button()
							.on_press(SidebarPageMessage::CloseCreateNewProject.into())
					]
					.align_items(Alignment::Center)
				)
				.max_width(300.0)
			)
			.width(Length::Fill)
			.align_x(Horizontal::Center)
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
				overview_button(app.content_page.is_overview_page()),
				partial_horizontal_seperator(),

				list,
	
				partial_horizontal_seperator(),
	
				create_new_project_element,
			]
			.height(Length::Fill)
			.spacing(SPACING_AMOUNT)
			.padding(HORIZONTAL_PADDING),
			
			container(settings_button())
				.align_y(Vertical::Bottom)
		]
		.width(Length::Fill)
		.height(Length::Fill)
		.spacing(SPACING_AMOUNT)
		.padding(Padding { top: SMALL_PADDING_AMOUNT, ..Padding::ZERO })
		.into()
	}
}