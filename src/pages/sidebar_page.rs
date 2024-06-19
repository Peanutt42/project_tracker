use iced::{alignment::Horizontal, theme, widget::{column, container, scrollable, scrollable::RelativeOffset, text_input, Column, Space}, Command, Element, Length, Padding};
use once_cell::sync::Lazy;
use crate::{core::DatabaseMessage, project_tracker::UiMessage};
use crate::components::{create_new_project_button, loading_screen, overview_button, partial_horizontal_seperator, project_preview, custom_project_preview, EDIT_PROJECT_NAME_TEXT_INPUT_ID, settings_button};
use crate::styles::{TextInputStyle, ScrollableStyle, scrollable_vertical_direction, LARGE_TEXT_SIZE, SMALL_PADDING_AMOUNT, PADDING_AMOUNT, SCROLLBAR_WIDTH, SPACING_AMOUNT};
use crate::project_tracker::ProjectTrackerApp;
use crate::core::{OrderedHashMap, ProjectId, generate_project_id, Project};

static SCROLLABLE_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);
static TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

#[derive(Debug, Clone)]
pub enum SidebarPageMessage {
	OpenCreateNewProject,
	CloseCreateNewProject,
	ChangeCreateNewProjectName(String),

	SidebarMoved(u16),

	EditProject(ProjectId),
	StopEditingProject,

	HoveringProject(ProjectId),
	StoppedHoveringProject,
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
	hovered_project_id: Option<ProjectId>,
	pub dividor_position: u16,
}

impl SidebarPage {
	pub fn new() -> Self {
		Self {
			create_new_project_name: None,
			project_being_edited: None,
			hovered_project_id: None,
			dividor_position: 300,
		}
	}

	fn project_preview_list<'a>(&'a self, projects: &'a OrderedHashMap<ProjectId, Project>, hovered_project_id: Option<ProjectId>, app: &'a ProjectTrackerApp) -> Element<'a, UiMessage> {
		let mut list: Vec<Element<UiMessage>> = projects.iter().enumerate()
			.map(|(i, project_id)| {
				let selected = match app.selected_project_id {
					Some(selected_project_id) => *project_id == selected_project_id,
					None => false,
				};
				let hovered = match hovered_project_id {
					Some(hovered_project_id) => *project_id == hovered_project_id,
					None => false,
				};
				let can_move_up = i != 0;
				let can_move_down = i != projects.len() - 1;
				let editing = match self.project_being_edited {
					Some(project_being_edited_id) => project_being_edited_id == *project_id,
					None => false,
				};
				project_preview(projects.get(project_id).unwrap(), *project_id, hovered, editing, can_move_up, can_move_down, selected)
			})
			.collect();

		if let Some(create_new_project_name) = &self.create_new_project_name {
			let project_name_text_input_element = container(
				text_input("New project name", create_new_project_name)
					.id(TEXT_INPUT_ID.clone())
					.size(LARGE_TEXT_SIZE)
					.on_input(|input| SidebarPageMessage::ChangeCreateNewProjectName(input).into())
					.on_submit(DatabaseMessage::CreateProject{
						project_id: generate_project_id(),
						name: create_new_project_name.clone()
					}.into())
					.style(theme::TextInput::Custom(Box::new(TextInputStyle)))
			)
			.width(Length::Fill)
			.align_x(Horizontal::Center)
			.into();

			list.push(custom_project_preview(None, false, false, false, false, 0.0, 0, 0, project_name_text_input_element, true));
		}

		// some space at the bottom so that the + button doesn't block any view to the last project
		list.push(Space::with_height(50.0).into());

		scrollable(
			Column::from_vec(list)
				.width(Length::Fill)
				.spacing(SPACING_AMOUNT)
    			.padding(Padding{ right: SMALL_PADDING_AMOUNT + SCROLLBAR_WIDTH, ..Padding::ZERO })
		)
		.id(SCROLLABLE_ID.clone())
		.width(Length::Fill)
		.height(Length::Fill)
		.style(theme::Scrollable::custom(ScrollableStyle))
		.direction(scrollable_vertical_direction())
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

			SidebarPageMessage::SidebarMoved(positon) => {
				self.dividor_position = positon;
				Command::none()
			},

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

			SidebarPageMessage::HoveringProject(project_id) => {
				self.hovered_project_id = Some(project_id);
				Command::none()
			},
			SidebarPageMessage::StoppedHoveringProject => {
				self.hovered_project_id = None;
				Command::none()
			},
		}
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<UiMessage> {
		let scrollbar_padding = Padding{ right: SMALL_PADDING_AMOUNT + SCROLLBAR_WIDTH, ..Padding::ZERO };

		let list: Element<UiMessage> = if let Some(database) = &app.database {
				self.project_preview_list(&database.projects, self.hovered_project_id, app)
			}
			else {
				container(loading_screen())
					.padding(scrollbar_padding)
					.into()
			};

		column![
			container(
				column![
					overview_button(app.content_page.is_overview_page()),
					partial_horizontal_seperator(),
				]
				.spacing(SPACING_AMOUNT)
			)
			.padding(scrollbar_padding),

			list,

			container(
				column![
					container(create_new_project_button(self.create_new_project_name.is_none()))
												.width(Length::Fill)
												.align_x(Horizontal::Right),

					partial_horizontal_seperator(),
					settings_button(app.content_page.is_settings_page()),
				]
				.spacing(SPACING_AMOUNT)
			)
			.padding(scrollbar_padding),
		]
		.width(Length::Fill)
		.height(Length::Fill)
		.spacing(SPACING_AMOUNT)
		.padding(Padding{ left: PADDING_AMOUNT, right: 0.0, top: PADDING_AMOUNT, bottom: PADDING_AMOUNT })
		.into()
	}
}
