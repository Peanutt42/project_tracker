use crate::{
	components::{
		cancel_search_tasks_button, color_palette, completion_bar, edit_color_palette_button,
		horizontal_scrollable, loading_screen, on_input, open_create_task_modal_button,
		project_context_menu_button, search_tasks_button, sort_dropdown_button, task_list,
		task_tag_button, ScalarAnimation, HORIZONTAL_SCROLLABLE_PADDING,
		LARGE_LOADING_SPINNER_SIZE,
	},
	core::{import_source_code_todos, IcedColorConversion, SortModeUI},
	icons::{icon_to_char, Bootstrap, BOOTSTRAP_FONT},
	pages,
	project_tracker::{self, ProjectTrackerApp},
	styles::{
		text_input_style_borderless, text_input_style_only_round_left, PADDING_AMOUNT,
		SMALL_SPACING_AMOUNT, SPACING_AMOUNT, TITLE_TEXT_SIZE,
	},
	DatabaseState, OptionalPreference, Preferences,
};
use chrono::{DateTime, Utc};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use iced::{
	alignment::{Alignment, Horizontal},
	keyboard,
	widget::{column, container, row, text, text_input, Row, Space},
	Color, Element,
	Length::Fill,
	Padding, Subscription,
};
use iced_aw::{drop_down, DropDown};
use project_tracker_core::{
	Database, DatabaseMessage, OrderedHashMap, Project, ProjectId, SerializableColor, SortMode,
	Task, TaskId, TaskTagId, TaskType,
};
use std::{collections::HashSet, path::PathBuf, sync::LazyLock, time::SystemTime};
use tracing::error;

static PROJECT_NAME_TEXT_INPUT_ID: LazyLock<text_input::Id> = LazyLock::new(text_input::Id::unique);
static SEARCH_TASKS_TEXT_INPUT_ID: LazyLock<text_input::Id> = LazyLock::new(text_input::Id::unique);

#[derive(Clone, Debug)]
pub enum Message {
	RefreshCachedTaskList,

	OpenSortModeDropdown,
	CloseSortModeDropdown,
	SetSortMode(SortMode),

	ShowContextMenu,
	HideContextMenu,
	OpenManageTaskTagsModal,

	OpenSearchTasks,
	CloseSearchTasks,
	ChangeSearchTasksFilter(String),

	ImportSourceCodeTodosDialog,
	ImportSourceCodeTodos {
		source_code_directory: PathBuf,
		source_code_todos: OrderedHashMap<TaskId, Task>,
	},
	ReimportSourceCodeTodos,
	ImportSourceCodeTodosDialogCanceled,

	ShowSourceCodeTodos(bool),

	ShowDoneTasks(bool),

	ToggleFilterTaskTag(TaskTagId),
	UnsetFilterTaskTag(TaskTagId),

	ShowColorPicker,
	HideColorPicker,
	ChangeProjectColor(Color),

	ConfirmDeleteProject,

	AnimateProgressbar,
}

impl From<Message> for pages::Message {
	fn from(value: Message) -> Self {
		pages::Message::ProjectPage(value)
	}
}

impl From<Message> for project_tracker::Message {
	fn from(value: Message) -> Self {
		pages::Message::ProjectPage(value).into()
	}
}

#[derive(Debug, Clone)]
pub struct CachedTaskList {
	pub todo: Vec<TaskId>,
	pub done: Vec<TaskId>,
	pub source_code_todo: Vec<TaskId>,
	cache_time: SystemTime,
}

impl CachedTaskList {
	pub fn new(todo: Vec<TaskId>, done: Vec<TaskId>, source_code_todo: Vec<TaskId>) -> Self {
		Self {
			todo,
			done,
			source_code_todo,
			cache_time: SystemTime::now(),
		}
	}

	pub fn generate(
		project: &Project,
		task_tag_filter: &HashSet<TaskTagId>,
		search_filter: &Option<String>,
		sort_unspecified_tasks_at_bottom: bool,
	) -> Self {
		let mut todo_list = Vec::new();
		let mut done_list = Vec::new();
		let mut source_code_todo_list = Vec::new();

		match search_filter.as_ref().and_then(|search_filter| {
			if search_filter.is_empty() {
				None // pretend that no filter is enabled when fitler is empty
			} else {
				Some(search_filter)
			}
		}) {
			Some(search_filter) => {
				let mut todo_score_map = Vec::new();
				let mut done_score_map = Vec::new();
				let mut source_code_todo_score_map = Vec::new();
				for (task_id, task, task_type) in project.iter() {
					if task.matches_filter(task_tag_filter) {
						let task_name_match =
							SkimMatcherV2::default().fuzzy_match(&task.name, search_filter);
						let task_description_match =
							SkimMatcherV2::default().fuzzy_match(&task.description, search_filter);

						let task_match = match (task_name_match, task_description_match) {
							(Some(task_name_match), Some(task_description_match)) => {
								Some(task_name_match + task_description_match)
							}
							_ => task_name_match.or(task_description_match),
						};

						if let Some(task_match) = task_match {
							match task_type {
								TaskType::Todo => todo_score_map.push((task_id, task_match)),
								TaskType::Done => done_score_map.push((task_id, task_match)),
								TaskType::SourceCodeTodo => {
									source_code_todo_score_map.push((task_id, task_match))
								}
							}
						}
					}
				}

				let sort_compare = |a: &(TaskId, i64), b: &(TaskId, i64)| {
					let (_task_id_a, score_a) = a;
					let (_task_id_b, score_b) = b;
					score_b.cmp(score_a)
				};

				todo_score_map.sort_unstable_by(sort_compare);
				done_score_map.sort_unstable_by(sort_compare);
				source_code_todo_score_map.sort_unstable_by(sort_compare);

				todo_list = todo_score_map
					.into_iter()
					.map(|(task_id, _match)| task_id)
					.collect();
				done_list = done_score_map
					.into_iter()
					.map(|(task_id, _match)| task_id)
					.collect();
				source_code_todo_list = source_code_todo_score_map
					.into_iter()
					.map(|(task_id, _match)| task_id)
					.collect();
			}
			None => {
				for (task_id, task, task_type) in project.iter() {
					if task.matches_filter(task_tag_filter) {
						match task_type {
							TaskType::Todo => todo_list.push(task_id),
							TaskType::Done => done_list.push(task_id),
							TaskType::SourceCodeTodo => source_code_todo_list.push(task_id),
						}
					}
				}

				project
					.sort_mode
					.sort(project, &mut todo_list, sort_unspecified_tasks_at_bottom);
				project
					.sort_mode
					.sort(project, &mut done_list, sort_unspecified_tasks_at_bottom);
				project.sort_mode.sort(
					project,
					&mut source_code_todo_list,
					sort_unspecified_tasks_at_bottom,
				);
			}
		}

		Self::new(todo_list, done_list, source_code_todo_list)
	}
}

#[derive(Debug, Clone)]
pub struct Page {
	pub project_id: ProjectId,
	pub cached_task_list: CachedTaskList,
	show_done_tasks: bool,
	show_source_code_todos: bool,
	show_color_picker: bool,
	pub filter_task_tags: HashSet<TaskTagId>,
	search_tasks_filter: Option<String>,
	progressbar_animation: ScalarAnimation,
	previous_project_progress: f32,
	show_context_menu: bool,
	show_sort_mode_dropdown: bool,
	pub importing_source_code_todos: bool,
}

impl Page {
	pub fn new(
		project_id: ProjectId,
		project: &Project,
		preferences: &Option<Preferences>,
	) -> Self {
		let cached_task_list = CachedTaskList::generate(
			project,
			&HashSet::new(),
			&None,
			preferences.sort_unspecified_tasks_at_bottom(),
		);

		Self {
			project_id,
			cached_task_list,
			show_done_tasks: false,
			show_source_code_todos: true,
			show_color_picker: false,
			filter_task_tags: HashSet::new(),
			search_tasks_filter: None,
			progressbar_animation: ScalarAnimation::Idle,
			previous_project_progress: project.get_completion_percentage(),
			show_context_menu: false,
			show_sort_mode_dropdown: false,
			importing_source_code_todos: false,
		}
	}

	pub fn filtering_tasks(&self) -> bool {
		!self.filter_task_tags.is_empty()
			|| self
				.search_tasks_filter
				.as_ref()
				.map(|search_filter| !search_filter.is_empty())
				.unwrap_or(false)
	}

	pub fn update(
		&mut self,
		message: Message,
		database: Option<&Database>,
		preferences: &Option<Preferences>,
	) -> pages::Action {
		let command = match message {
			Message::RefreshCachedTaskList => pages::Action::None,

			Message::OpenSortModeDropdown => {
				self.show_sort_mode_dropdown = true;
				pages::Action::None
			}
			Message::CloseSortModeDropdown => {
				self.show_sort_mode_dropdown = false;
				pages::Action::None
			}
			Message::SetSortMode(new_sort_mode) => {
				self.show_sort_mode_dropdown = false;
				DatabaseMessage::ChangeProjectSortMode {
					project_id: self.project_id,
					new_sort_mode,
				}
				.into()
			}

			Message::ShowContextMenu => {
				self.show_context_menu = true;
				pages::Action::None
			}
			Message::HideContextMenu => {
				self.show_context_menu = false;
				pages::Action::None
			}
			Message::OpenManageTaskTagsModal => {
				self.show_context_menu = false;
				pages::Action::OpenManageTaskTagsModal(self.project_id)
			}

			Message::OpenSearchTasks => {
				self.search_tasks_filter = Some(String::new());
				if let Some(database) = database {
					self.generate_cached_task_list(database, preferences);
				}
				text_input::focus(SEARCH_TASKS_TEXT_INPUT_ID.clone()).into()
			}
			Message::CloseSearchTasks => {
				self.search_tasks_filter = None;
				if let Some(database) = database {
					self.generate_cached_task_list(database, preferences);
				}
				pages::Action::None
			}
			Message::ChangeSearchTasksFilter(new_filter) => {
				self.search_tasks_filter = Some(new_filter);
				if let Some(database) = database {
					self.generate_cached_task_list(database, preferences);
				}
				pages::Action::None
			}

			Message::ImportSourceCodeTodosDialog => {
				self.show_context_menu = false;
				self.importing_source_code_todos = true;
				iced::Task::perform(
					Self::pick_todo_source_code_folder_dialog(),
					|source_code_todos| match source_code_todos {
						Some((source_code_directory, source_code_todos)) => {
							Message::ImportSourceCodeTodos {
								source_code_directory,
								source_code_todos,
							}
							.into()
						}
						None => Message::ImportSourceCodeTodosDialogCanceled.into(),
					},
				)
				.into()
			}
			Message::ImportSourceCodeTodosDialogCanceled => {
				self.importing_source_code_todos = false;
				pages::Action::None
			}
			Message::ImportSourceCodeTodos {
				source_code_directory,
				source_code_todos,
			} => {
				self.importing_source_code_todos = false;
				DatabaseMessage::ImportSourceCodeTodos {
					project_id: self.project_id,
					source_code_directory,
					source_code_todo_tasks: source_code_todos,
				}
				.into()
			}
			Message::ReimportSourceCodeTodos => {
				match database
					.and_then(|db| db.get_project(&self.project_id))
					.and_then(|project| project.source_code_directory.clone())
				{
					Some(source_code_directory) => {
						self.importing_source_code_todos = true;
						let source_code_directory_clone = source_code_directory.clone();

						iced::Task::perform(
							async move { import_source_code_todos(source_code_directory_clone) },
							move |source_code_todos| {
								Message::ImportSourceCodeTodos {
									source_code_directory: source_code_directory.clone(),
									source_code_todos,
								}
								.into()
							},
						)
						.into()
					}
					None => {
						self.update(Message::ImportSourceCodeTodosDialog, database, preferences)
					}
				}
			}

			Message::ShowSourceCodeTodos(show) => {
				self.show_source_code_todos = show;
				pages::Action::None
			}

			Message::ShowDoneTasks(show) => {
				self.show_done_tasks = show;
				pages::Action::None
			}

			Message::ToggleFilterTaskTag(task_tag_id) => {
				if self.filter_task_tags.contains(&task_tag_id) {
					self.filter_task_tags.remove(&task_tag_id);
				} else {
					self.filter_task_tags.insert(task_tag_id);
				}
				if let Some(database) = database {
					self.generate_cached_task_list(database, preferences);
				}
				pages::Action::None
			}
			Message::UnsetFilterTaskTag(task_tag_id) => {
				self.filter_task_tags.remove(&task_tag_id);
				if let Some(database) = database {
					self.generate_cached_task_list(database, preferences);
				}
				pages::Action::None
			}

			Message::ShowColorPicker => {
				self.show_color_picker = true;
				pages::Action::None
			}
			Message::HideColorPicker => {
				self.show_color_picker = false;
				pages::Action::None
			}
			Message::ChangeProjectColor(new_color) => {
				self.show_color_picker = false;
				DatabaseMessage::ChangeProjectColor {
					project_id: self.project_id,
					new_color: SerializableColor::from_iced_color(new_color),
				}
				.into()
			}

			Message::ConfirmDeleteProject => {
				self.show_context_menu = false;

				let project_name = database
					.as_ref()
					.and_then(|db| {
						db.get_project(&self.project_id)
							.map(|project| project.name.clone())
					})
					.unwrap_or({
						error!("invalid project_id: doesnt exist in database!");
						"<invalid project id>".to_string()
					});

				pages::Action::ConfirmDeleteProject {
					project_id: self.project_id,
					project_name,
				}
			}

			Message::AnimateProgressbar => {
				self.progressbar_animation.update();
				pages::Action::None
			}
		};

		if let Some(database_ref) = database {
			let cache_date_time: DateTime<Utc> = self.cached_task_list.cache_time.into();
			if cache_date_time < *database_ref.last_changed_time() {
				self.generate_cached_task_list(database_ref, preferences);
			}
		}

		command
	}

	pub fn subscription(&self) -> Subscription<Message> {
		Subscription::batch([
			self.progressbar_animation
				.subscription()
				.map(|_| Message::AnimateProgressbar),
			keyboard::on_key_press(|key, modifiers| match key.as_ref() {
				keyboard::Key::Character("f") if modifiers.command() => {
					Some(Message::OpenSearchTasks)
				}
				_ => None,
			}),
		])
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<'a, project_tracker::Message> {
		match &app.database {
			DatabaseState::Loaded(database) => {
				match database.get_project(&self.project_id) {
					Some(project) => column![
						self.project_details_view(project),
						task_list(
							self.project_id,
							project,
							&self.cached_task_list,
							&app.task_ui_id_map,
							app.preferences.code_editor(),
							app.dragged_task,
							app.just_minimal_dragging,
							app.sidebar_page.task_dropzone_hovered,
							self.show_done_tasks,
							self.show_source_code_todos,
							self.importing_source_code_todos
						),
					]
					// .spacing(SPACING_AMOUNT) this is not needed since every task in the list has a SPACING_AMOUNT height dropzone
					.width(Fill)
					.height(Fill)
					.into(),
					None => {
						error!("invalid project_id inside project_page: doesnt exist in database!");
						text("<Invalid ProjectId>").into()
					}
				}
			}
			_ => container(loading_screen(LARGE_LOADING_SPINNER_SIZE))
				.center(Fill)
				.into(),
		}
	}

	fn project_details_view<'a>(
		&'a self,
		project: &'a Project,
	) -> Element<'a, project_tracker::Message> {
		let project_name: Element<project_tracker::Message> =
			text_input("New project name", &project.name)
				.id(PROJECT_NAME_TEXT_INPUT_ID.clone())
				.size(TITLE_TEXT_SIZE)
				.on_input(|edited_name| {
					DatabaseMessage::ChangeProjectName {
						project_id: self.project_id,
						new_name: edited_name.clone(),
					}
					.into()
				})
				.style(|t, s| text_input_style_borderless(t, s, false))
				.into(); //text_input_style_default),

		let show_color_picker_button = edit_color_palette_button(
			project.color.to_iced_color(),
			self.show_color_picker,
			if self.show_color_picker {
				Message::HideColorPicker.into()
			} else {
				Message::ShowColorPicker.into()
			},
		);

		let color_picker = DropDown::new(
			show_color_picker_button,
			color_palette(project.color.to_iced_color(), |c| {
				Message::ChangeProjectColor(c).into()
			}),
			self.show_color_picker,
		)
		.width(Fill)
		.alignment(drop_down::Alignment::End)
		.on_dismiss(Message::HideColorPicker.into());

		let task_tags_list: Vec<Element<project_tracker::Message>> = project
			.task_tags
			.iter()
			.map(|(tag_id, tag)| {
				task_tag_button(tag, self.filter_task_tags.contains(&tag_id))
					.on_press(Message::ToggleFilterTaskTag(tag_id).into())
					.into()
			})
			.collect();

		let search_tasks_element: Element<project_tracker::Message> =
			match &self.search_tasks_filter {
				Some(search_tasks_filter) => row![
					on_input(
						text_input("Search tasks...", search_tasks_filter)
							.id(SEARCH_TASKS_TEXT_INPUT_ID.clone())
							.icon(text_input::Icon {
								font: BOOTSTRAP_FONT,
								code_point: icon_to_char(Bootstrap::Search),
								size: None,
								spacing: SMALL_SPACING_AMOUNT as f32,
								side: text_input::Side::Left,
							})
							.on_input(|new_search_filter| {
								Message::ChangeSearchTasksFilter(new_search_filter).into()
							})
							.style(text_input_style_only_round_left)
					)
					.on_esc(Message::CloseSearchTasks.into()),
					cancel_search_tasks_button(),
				]
				.into(),
				None => search_tasks_button().into(),
			};

		let quick_actions: Element<project_tracker::Message> = row![
			search_tasks_element,
			project_context_menu_button(self.show_context_menu),
		]
		.spacing(SPACING_AMOUNT)
		.into();

		let spacer = || Space::new(Fill, SPACING_AMOUNT);

		column![
			column![row![color_picker, project_name, quick_actions]
				.spacing(SPACING_AMOUNT)
				.align_y(Alignment::Center)]
			.width(Fill),
			spacer(),
			row![
				container("Tags:").padding(HORIZONTAL_SCROLLABLE_PADDING),
				horizontal_scrollable(Row::with_children(task_tags_list).spacing(SPACING_AMOUNT))
					.width(Fill),
				container(sort_dropdown_button(
					self.show_sort_mode_dropdown,
					project.sort_mode
				))
				.padding(HORIZONTAL_SCROLLABLE_PADDING),
			]
			.spacing(SPACING_AMOUNT)
			.align_y(Alignment::Center),
			row![
				text(format!(
					"{}/{} finished ({}%)",
					project.done_tasks.len(),
					project.total_tasks(),
					(project.get_completion_percentage() * 100.0).round()
				))
				.width(Fill),
				container(open_create_task_modal_button())
					.width(Fill)
					.align_x(Horizontal::Right),
			]
			.width(Fill)
			.align_y(Alignment::Center),
			spacer(),
			completion_bar(
				self.progressbar_animation
					.get_value()
					.unwrap_or(self.previous_project_progress)
			),
		]
		.padding(Padding {
			top: PADDING_AMOUNT,
			bottom: 0.0, // task_list already has padding on the top due to dropzone padding/spacing
			left: PADDING_AMOUNT,
			right: PADDING_AMOUNT,
		})
		.into()
	}

	fn start_progressbar_animation(&mut self, start_percentage: f32, target_percentage: f32) {
		self.progressbar_animation =
			ScalarAnimation::start(start_percentage, target_percentage, 0.125);
	}

	pub fn generate_cached_task_list(
		&mut self,
		database: &Database,
		preferences: &Option<Preferences>,
	) {
		if let Some(project) = database.get_project(&self.project_id) {
			self.cached_task_list = CachedTaskList::generate(
				project,
				&self.filter_task_tags,
				&self.search_tasks_filter,
				preferences.sort_unspecified_tasks_at_bottom(),
			);

			// update progress bar animation
			let new_project_progress = project.get_completion_percentage();
			if new_project_progress != self.previous_project_progress {
				self.start_progressbar_animation(
					self.previous_project_progress,
					new_project_progress,
				);
				self.previous_project_progress = new_project_progress;
			}
		}
	}

	async fn pick_todo_source_code_folder_dialog() -> Option<(PathBuf, OrderedHashMap<TaskId, Task>)>
	{
		let file_dialog_result = rfd::AsyncFileDialog::new()
			.set_title("Import Todos from source code folder")
			.pick_folder()
			.await;

		file_dialog_result.map(|folder_handle| {
			let pathbuf = folder_handle.path().to_path_buf();
			(pathbuf.clone(), import_source_code_todos(pathbuf))
		})
	}
}
