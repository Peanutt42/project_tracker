use iced::{widget::{button, column, row, text, text_input, Column}, Alignment, Color, Element, Length::Fill, Task};
use iced_aw::card;
use once_cell::sync::Lazy;
use crate::{components::{cancel_create_new_task_tag_button, color_palette, color_palette_item_button, create_new_task_tags_button, delete_task_tag_button, unfocusable}, core::{Database, DatabaseMessage, Project, ProjectId, TaskTag, TaskTagId}, project_tracker::UiMessage, styles::{card_style, hidden_secondary_button_style, text_input_style_only_round_left, LARGE_TEXT_SIZE, SMALL_SPACING_AMOUNT}, ProjectTrackerApp};

static CREATE_NEW_TASK_TAG_NAME_TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

#[derive(Debug, Clone)]
pub enum ManageTaskTagsModalMessage {
	Open {
		project_id: ProjectId
	},
	OpenCreateNewTaskTag,
	CloseCreateNewTaskTag,
	ChangeCreateNewTaskTagName(String),
	EditTaskTagColor(TaskTagId),
	EditTaskTagName(TaskTagId),
	ChangeEditTaskTagName(String),
	ChangeTaskTagName,
	ChangeTaskTagColor(Color),
	StopEditTaskTagColor,
	StopEditTaskTagName,
	CreateNewTaskTag,
	DeleteTaskTag(TaskTagId),
	Close,
}

impl From<ManageTaskTagsModalMessage> for UiMessage {
	fn from(value: ManageTaskTagsModalMessage) -> Self {
		UiMessage::ManageTaskTagsModalMessage(value)
	}
}

pub enum ManageTaskTagsModal {
	Opened{
		project_id: ProjectId,
		create_new_task_tag: Option<String>,
		edit_task_tag_color_id: Option<TaskTagId>,
		edit_task_tag_name_id: Option<(TaskTagId, String)>,
	},
	Closed,
}

impl ManageTaskTagsModal {
	pub fn update<'a>(&'a mut self, message: ManageTaskTagsModalMessage, database: &'a mut Option<Database>) -> Task<UiMessage> {
		match message {
			ManageTaskTagsModalMessage::Open { project_id } => {
				*self = ManageTaskTagsModal::Opened {
					project_id,
					create_new_task_tag: None,
					edit_task_tag_color_id: None,
					edit_task_tag_name_id: None
				};
				Task::none()
			},
			ManageTaskTagsModalMessage::OpenCreateNewTaskTag => {
				if let ManageTaskTagsModal::Opened { create_new_task_tag, .. } = self {
					*create_new_task_tag = Some(String::new());
				}
				Task::batch([
					text_input::focus(CREATE_NEW_TASK_TAG_NAME_TEXT_INPUT_ID.clone()),
					self.update(ManageTaskTagsModalMessage::StopEditTaskTagColor, database),
					self.update(ManageTaskTagsModalMessage::StopEditTaskTagName, database),
				])
			},
			ManageTaskTagsModalMessage::CloseCreateNewTaskTag => {
				if let ManageTaskTagsModal::Opened { create_new_task_tag, .. } = self {
					*create_new_task_tag = None;
				}
				Task::none()
			},
			ManageTaskTagsModalMessage::ChangeCreateNewTaskTagName(new_name) => {
				if let ManageTaskTagsModal::Opened { create_new_task_tag, .. } = self {
					*create_new_task_tag = Some(new_name);
				}
				Task::none()
			},
			ManageTaskTagsModalMessage::EditTaskTagColor(task_tag_id) => {
				if let ManageTaskTagsModal::Opened { edit_task_tag_color_id, .. } = self {
					*edit_task_tag_color_id = Some(task_tag_id);
				}
				self.update(ManageTaskTagsModalMessage::StopEditTaskTagName, database)
			},
			ManageTaskTagsModalMessage::EditTaskTagName(task_tag_id) => {
				if let ManageTaskTagsModal::Opened { edit_task_tag_name_id, project_id, .. } = self {
					let task_tag_name = database.as_ref().and_then(|database| {
						database
							.get_project(project_id)
							.and_then(|project| {
								project.task_tags.get(&task_tag_id).map(|task_tag| task_tag.name.clone())
							})
					}).unwrap_or_default();
					*edit_task_tag_name_id = Some((task_tag_id, task_tag_name));
				}
				self.update(ManageTaskTagsModalMessage::StopEditTaskTagColor, database)
			},
			ManageTaskTagsModalMessage::ChangeTaskTagColor(new_color) => {
				if let ManageTaskTagsModal::Opened { project_id, edit_task_tag_color_id: Some(edit_task_tag_id), .. } = self {
					if let Some(db) = database {
						db.update(DatabaseMessage::ChangeTaskTagColor {
							project_id: *project_id,
							task_tag_id: *edit_task_tag_id,
							new_color: new_color.into()
						});
						return self.update(ManageTaskTagsModalMessage::StopEditTaskTagColor, database);
					}
				}
				self.update(ManageTaskTagsModalMessage::StopEditTaskTagColor, database)
			},
			ManageTaskTagsModalMessage::ChangeEditTaskTagName(new_name) => {
				if let ManageTaskTagsModal::Opened { edit_task_tag_name_id: Some((_edit_task_tag_id, edit_task_tag_name)), .. } = self {
					*edit_task_tag_name = new_name;
				}
				Task::none()
			},
			ManageTaskTagsModalMessage::ChangeTaskTagName => {
				if let ManageTaskTagsModal::Opened { project_id, edit_task_tag_name_id: Some((edit_task_tag_id, new_name)), .. } = self {
					if let Some(db) = database {
						db.update(DatabaseMessage::ChangeTaskTagName {
							project_id: *project_id,
							task_tag_id: *edit_task_tag_id,
							new_name: std::mem::take(new_name)
						});
						return self.update(ManageTaskTagsModalMessage::StopEditTaskTagName, database);
					}
				}
				self.update(ManageTaskTagsModalMessage::StopEditTaskTagName, database)
			},
			ManageTaskTagsModalMessage::StopEditTaskTagColor => {
				if let ManageTaskTagsModal::Opened { edit_task_tag_color_id, .. } = self {
					*edit_task_tag_color_id = None;
				}
				Task::none()
			},
			ManageTaskTagsModalMessage::StopEditTaskTagName => {
				if let ManageTaskTagsModal::Opened { edit_task_tag_name_id, .. } = self {
					*edit_task_tag_name_id = None;
				}
				Task::none()
			},
			ManageTaskTagsModalMessage::CreateNewTaskTag => {
				if let ManageTaskTagsModal::Opened { project_id, create_new_task_tag: Some(new_task_tag_name), .. } = self {
					if let Some(db) = database {
						db.update(DatabaseMessage::CreateTaskTag {
							project_id: *project_id,
							task_tag_id: TaskTagId::generate(),
							task_tag: TaskTag::new(std::mem::take(new_task_tag_name), Color::WHITE.into())
						});
						return self.update(ManageTaskTagsModalMessage::CloseCreateNewTaskTag, database);
					}
				}
				self.update(ManageTaskTagsModalMessage::CloseCreateNewTaskTag, database)
			},
			ManageTaskTagsModalMessage::DeleteTaskTag(task_tag_id) => {
				if let ManageTaskTagsModal::Opened { project_id, .. } = self {
					if let Some(database) = database {
						database.update(DatabaseMessage::DeleteTaskTag {
							project_id: *project_id,
							task_tag_id
						});
					}
				}
				Task::none()
			},
			ManageTaskTagsModalMessage::Close => {
				*self = ManageTaskTagsModal::Closed;
				Task::none()
			},
		}
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Option<Element<UiMessage>> {
		match self {
			ManageTaskTagsModal::Opened { project_id, create_new_task_tag, edit_task_tag_color_id, edit_task_tag_name_id } => {
				app.database.as_ref().and_then(|db| db.get_project(project_id)).map(|project| {
					card(
						text(format!("Manage project '{}' task tags", project.name))
							.size(LARGE_TEXT_SIZE),

						self.tags_list_view(project, create_new_task_tag, edit_task_tag_color_id, edit_task_tag_name_id)
					)
					.style(card_style)
					.max_width(500.0)
					.close_size(LARGE_TEXT_SIZE)
					.on_close(ManageTaskTagsModalMessage::Close.into())
					.into()
				})
			},
			ManageTaskTagsModal::Closed => {
				None
			},
		}
	}

	fn tags_list_view<'a>(&'a self, project: &'a Project, create_new_task_tag: &'a Option<String>, edit_task_tag_color_id: &'a Option<TaskTagId>, edit_task_tag_name_id: &'a Option<(TaskTagId, String)>) -> Element<'a, UiMessage> {
		let mut tags_list = Vec::new();
		for (tag_id, tag) in project.task_tags.iter() {
			let show_color_palette = match edit_task_tag_color_id {
				Some(edit_task_tag_color_id) => tag_id == *edit_task_tag_color_id,
				None => false,
			};

			let edited_name = match edit_task_tag_name_id {
				Some((edit_task_tag_name_id, new_name)) if tag_id == *edit_task_tag_name_id => Some(new_name),
				_ => None,
			};

			let name_element: Element<UiMessage> = if let Some(edited_name) = edited_name {
				text_input("tag name", edited_name)
					.on_input(move |new_name| ManageTaskTagsModalMessage::ChangeEditTaskTagName(new_name).into())
					.on_submit(ManageTaskTagsModalMessage::ChangeTaskTagName.into())
					.style(text_input_style_only_round_left)
					.into()
			}
			else {
				button(
					text(&tag.name)
            			.width(Fill)
				)
				.on_press(ManageTaskTagsModalMessage::EditTaskTagName(tag_id).into())
				.style(hidden_secondary_button_style)
				.into()
			};

			tags_list.push(
				column![
					row![
						color_palette_item_button(
							tag.color.into(),
							false,
							if show_color_palette {
								ManageTaskTagsModalMessage::StopEditTaskTagColor.into()
							}
							else {
								ManageTaskTagsModalMessage::EditTaskTagColor(tag_id).into()
							}
						),
						name_element,
						delete_task_tag_button(tag_id),
					]
					.align_y(Alignment::Center)
					.spacing(SMALL_SPACING_AMOUNT)
				]
				.push_maybe(
					if show_color_palette {
						Some(color_palette(tag.color.into(), move |new_color| ManageTaskTagsModalMessage::ChangeTaskTagColor(new_color).into()))
					}
					else {
						None
					}
				)
				.into()
			);
		}

		if let Some(create_new_task_tag_name) = create_new_task_tag {
			tags_list.push(
				row![
					unfocusable(
						text_input("New tag name", create_new_task_tag_name)
							.id(CREATE_NEW_TASK_TAG_NAME_TEXT_INPUT_ID.clone())
							.on_input(|new_name| ManageTaskTagsModalMessage::ChangeCreateNewTaskTagName(new_name).into())
							.on_submit(ManageTaskTagsModalMessage::CreateNewTaskTag.into())
							.style(text_input_style_only_round_left),

						ManageTaskTagsModalMessage::CloseCreateNewTaskTag.into()
					),

					cancel_create_new_task_tag_button(),
				].into()
			);
		}
		else {
			tags_list.push(create_new_task_tags_button().into());
		}

		Column::with_children(tags_list)
			.spacing(SMALL_SPACING_AMOUNT)
			.into()
	}
}