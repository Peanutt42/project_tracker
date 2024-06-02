use iced::{alignment::{Alignment, Horizontal}, theme, widget::{column, container, row, scrollable, scrollable::RelativeOffset, text_input, Column, Space}, Command, Element, Length, Padding};
use iced_aw::{floating_element, floating_element::Anchor};
use once_cell::sync::Lazy;
use crate::{project_tracker::UiMessage, styles::{LARGE_TEXT_SIZE, PADDING_AMOUNT}};
use crate::components::{cancel_button, create_new_project_button, loading_screen, overview_button, partial_horizontal_seperator, project_preview, custom_project_preview, EDIT_PROJECT_NAME_TEXT_INPUT_ID, settings_button};
use crate::styles::{TextInputStyle, SPACING_AMOUNT};
use crate::project_tracker::ProjectTrackerApp;
use crate::core::{OrderedHashMap, ProjectId, generate_project_id, Project};

static SCROLLABLE_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);
static TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

#[derive(Debug, Clone)]
pub enum SidebarPageMessage {
	OpenCreateNewProject,
	CloseCreateNewProject,
	ChangeCreateNewProjectName(String),

	EditProject(ProjectId),
	StopEditingProject,
}

impl From<SidebarPageMessage> for UiMessage {
	fn from(value: SidebarPageMessage) -> Self {
		UiMessage::SidebarPageMessage(value)
	}
}

#[derive(Debug, Clone)]
pub struct SidebarPage {
	create_new_project_name: Option<String>,
	pub project_being_edited: Option<ProjectId>,
}

impl SidebarPage {
	pub fn new() -> Self {
		Self {
			create_new_project_name: None,
			project_being_edited: None,
		}
	}

	fn project_preview_list<'a>(&'a self, projects: &'a OrderedHashMap<ProjectId, Project>, app: &'a ProjectTrackerApp) -> Element<'a, UiMessage> {
		let mut list: Vec<Element<UiMessage>> = projects.iter().enumerate()
			.map(|(i, project_id)| {
				let selected = match app.selected_project_id {
					Some(selected_project_id) => *project_id == selected_project_id,
					None => false,
				};
				let can_move_up = i != 0;
				let can_move_down = i != projects.len() - 1;
				let editing = match self.project_being_edited {
					Some(project_being_edited_id) => project_being_edited_id == *project_id,
					None => false,
				};
				project_preview(projects.get(project_id).unwrap(), *project_id, editing, can_move_up, can_move_down, selected)
			})
			.collect();

		let create_new_project_element = if let Some(create_new_project_name) = &self.create_new_project_name {
			let project_name_text_input_element = container(
				row![
					text_input("New project name", create_new_project_name)
						.id(TEXT_INPUT_ID.clone())
						.size(LARGE_TEXT_SIZE)
						.on_input(|input| SidebarPageMessage::ChangeCreateNewProjectName(input).into())
						.on_submit(UiMessage::CreateProject{ project_id: generate_project_id(), project_name: create_new_project_name.clone() })
						.style(theme::TextInput::Custom(Box::new(TextInputStyle))),
	
					cancel_button()
						.on_press(SidebarPageMessage::CloseCreateNewProject.into())
				]
				.align_items(Alignment::Center)
			)
			.width(Length::Fill)
			.align_x(Horizontal::Center)
			.into();

			custom_project_preview(None, false, false, false, 0.0, 0, 0, project_name_text_input_element, true)
		}
		else {
			column![].into()
		};

		list.push(create_new_project_element);

		// some space at the bottom so that the + button doesn't block any view to the last project
		list.push(Space::with_height(50.0).into());
		
		scrollable(
			Column::from_vec(list)
				.width(Length::Fill)
				.spacing(SPACING_AMOUNT)
		)
		.id(SCROLLABLE_ID.clone())
		.width(Length::Fill)
		.height(Length::Fill)
		.into()
	}

	pub fn update(&mut self, message: SidebarPageMessage) -> Command<UiMessage> {
		match message {
			SidebarPageMessage::OpenCreateNewProject => {
				self.create_new_project_name = Some(String::new());
				Command::batch([
					self.update(SidebarPageMessage::StopEditingProject),
					text_input::focus(TEXT_INPUT_ID.clone()),
					scrollable::snap_to(SCROLLABLE_ID.clone(), RelativeOffset::END),
				])
			},
			SidebarPageMessage::CloseCreateNewProject => { self.create_new_project_name = None; Command::none() },
			SidebarPageMessage::ChangeCreateNewProjectName(new_project_name) => { self.create_new_project_name = Some(new_project_name); Command::none() },

			SidebarPageMessage::EditProject(project_id) => {
				self.project_being_edited = Some(project_id);
				Command::batch([
					self.update(SidebarPageMessage::CloseCreateNewProject),
					text_input::focus(EDIT_PROJECT_NAME_TEXT_INPUT_ID.clone())
				])
			},
			SidebarPageMessage::StopEditingProject => {
				self.project_being_edited = None;
				Command::none()
			},
		}		
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<UiMessage> {
		let list: Element<UiMessage> =
			if let Some(database) = &app.database {
				self.project_preview_list(&database.projects, app)
			}
			else {
				loading_screen()
			};

		let create_new_project_button: Element<UiMessage> = if self.create_new_project_name.is_some() {
			column![].into()
		}
		else {
			create_new_project_button().into()
		};

		column![
			overview_button(app.content_page.is_overview_page()),
			
			partial_horizontal_seperator(),

			floating_element(
				list,
				create_new_project_button
			)
			.anchor(Anchor::SouthEast)
			.offset(SPACING_AMOUNT as f32),
			
			partial_horizontal_seperator(),

			settings_button(app.content_page.is_settings_page()),
		]
		.width(Length::Fill)
		.height(Length::Fill)
		.spacing(SPACING_AMOUNT)
		.padding(Padding::new(PADDING_AMOUNT))
		.into()
	}
}