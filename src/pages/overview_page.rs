use std::{collections::{BTreeMap, HashMap}, time::SystemTime};
use chrono::{Days, NaiveDate};
use iced::{widget::{column, container, row, text, Column}, Element, Length::Fill, Padding};
use iced_aw::date_picker::Date;
use crate::{components::{days_left_widget, open_project_button, task_widget, vertical_scrollable}, core::{Database, OptionalPreference, Preferences, ProjectId, SerializableDate, SortMode, Task, TaskId}, pages::{ContentPageMessage, StopwatchPage}, project_tracker::Message, styles::{rounded_container_style, PADDING_AMOUNT, SMALL_HORIZONTAL_PADDING, SPACING_AMOUNT}, ProjectTrackerApp};


#[derive(Debug, Clone)]
pub struct OverviewPage {
	overdue_tasks: BTreeMap<SerializableDate, HashMap<ProjectId, Vec<TaskId>>>, // sorted by due date, then by project
	today_tasks: HashMap<ProjectId, Vec<TaskId>>, // sorted by est. needed time
	tomorrow_tasks: HashMap<ProjectId, Vec<TaskId>>, // sorted by est. needed time
	cache_time: SystemTime, // TODO: implement regeneration
}

#[derive(Debug, Clone)]
pub enum OverviewPageMessage {
	RefreshCachedTaskList,
}

impl From<OverviewPageMessage> for Message {
	fn from(value: OverviewPageMessage) -> Self {
		ContentPageMessage::OverviewPageMessage(value).into()
	}
}

impl OverviewPage {
	pub fn new(database: &Option<Database>, preferences: &Option<Preferences>) -> Self {
		let mut overdue_tasks: BTreeMap<SerializableDate, HashMap<ProjectId, Vec<TaskId>>> = BTreeMap::new();
		let mut today_tasks: HashMap<ProjectId, Vec<TaskId>> = HashMap::new();
		let mut tomorrow_tasks: HashMap<ProjectId, Vec<TaskId>> = HashMap::new();

		if let Some(database) = &database {
			let today: NaiveDate = Date::today().into();
			let today_date: Date = today.into();

			let tomorrow = today.checked_add_days(Days::new(1));
			let tomorrow_date: Option<Date> = tomorrow.map(|date| date.into());

			for (project_id, project) in database.projects().iter() {
				let mut cache_overdue_tasks = |task_id: TaskId, task: &Task| {
					if let Some(due_date) = &task.due_date {
						if *due_date < today_date.into() {
							overdue_tasks.entry(*due_date)
								.or_default()
								.entry(project_id)
								.or_default()
								.push(task_id);
						}
					}
				};
				let mut cache_today_tasks = |task_id: TaskId, task: &Task| {
					if let Some(due_date) = &task.due_date {
						if *due_date == today_date.into() {
							today_tasks.entry(project_id)
								.or_default()
								.push(task_id);
						}
					}
				};
				let mut cache_tomorrow_tasks = |task_id: TaskId, task: &Task| {
					if let Some(tomorrow_date) = tomorrow_date {
						if let Some(due_date) = &task.due_date {
							if *due_date == tomorrow_date.into() {
								tomorrow_tasks.entry(project_id)
									.or_default()
									.push(task_id);
							}
						}
					}
				};

				for (task_id, task) in project.todo_tasks.iter() {
					cache_overdue_tasks(task_id, task);
					cache_today_tasks(task_id, task);
					cache_tomorrow_tasks(task_id, task);
				}
				for (task_id, task) in project.source_code_todos.iter() {
					cache_overdue_tasks(*task_id, task);
					cache_today_tasks(*task_id, task);
					cache_tomorrow_tasks(*task_id, task);
				}
			}

			let sort_unspecified_tasks_at_bottom = preferences.sort_unspecified_tasks_at_bottom();
			for (project_id, tasks) in today_tasks.iter_mut() {
				let project = database.get_project(project_id).unwrap();
				SortMode::NeededTime.sort(project, tasks, sort_unspecified_tasks_at_bottom);
			}
			for (project_id, tasks) in tomorrow_tasks.iter_mut() {
				let project = database.get_project(project_id).unwrap();
				SortMode::NeededTime.sort(project, tasks, sort_unspecified_tasks_at_bottom);
			}
		}

		Self {
			overdue_tasks,
			today_tasks,
			tomorrow_tasks,
			cache_time: SystemTime::now(),
		}
	}

	pub fn update(&mut self, message: OverviewPageMessage, database: &Option<Database>, preferences: &Option<Preferences>) {
		match message {
			OverviewPageMessage::RefreshCachedTaskList => {
				if let Some(database_ref) = database {
					if self.cache_time < *database_ref.last_changed_time() {
						*self = Self::new(database, preferences);
					}
				}
			},
		}
	}

	// TODO: collapsable overdue, today, tomorrow sections
	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<'a, Message> {
		let overdue_tasks_len: usize = self.overdue_tasks.values()
			.map(|tasks|
				tasks.values()
					.map(|tasks| tasks.len())
					.sum::<usize>()
			)
			.sum();

		let today_tasks_len: usize = self.today_tasks.values()
			.map(|tasks| tasks.len())
			.sum();

		let tomorrow_tasks_len: usize = self.today_tasks.values()
					.map(|tasks| tasks.len())
					.sum();

		container(
			vertical_scrollable(
				Column::new()
					.push_maybe(if self.overdue_tasks.is_empty() {
						None
					} else {
						Some(column![
							row![
								text("Overdue"),
								container(text(overdue_tasks_len.to_string()))
									.padding(SMALL_HORIZONTAL_PADDING)
									.style(rounded_container_style)
							]
							.spacing(SPACING_AMOUNT),
							Column::with_children(self.overdue_tasks.iter()
								.map(|(date, tasks)| {
									column![
										days_left_widget(*date, false),
										Self::view_tasks(tasks, app),
									]
									.spacing(SPACING_AMOUNT)
									.padding(Padding::default().left(PADDING_AMOUNT))
									.into()
								}))
								.spacing(SPACING_AMOUNT)
						]
						.spacing(SPACING_AMOUNT))
					})
					.push_maybe(Self::view_tasks_for_day("Today", today_tasks_len, &self.today_tasks, app))
					.push_maybe(Self::view_tasks_for_day("Tomorrow", tomorrow_tasks_len, &self.tomorrow_tasks, app))
					.width(Fill)
					.spacing(SPACING_AMOUNT)
					.padding(PADDING_AMOUNT)
			)
		)
		.width(Fill)
		.height(Fill)
		.into()
	}

	fn view_tasks_for_day<'a>(time_label: &'static str, task_count: usize, tasks: &'a HashMap<ProjectId, Vec<TaskId>>, app: &'a ProjectTrackerApp)
		-> Option<Element<'a, Message>>
	{
		if tasks.is_empty() {
			None
		} else {
			Some(
				column![
					row![
						text(time_label),
						container(text(task_count.to_string()))
							.padding(SMALL_HORIZONTAL_PADDING)
							.style(rounded_container_style)
					]
					.spacing(SPACING_AMOUNT),
					Self::view_tasks(tasks, app),
				]
				.spacing(SPACING_AMOUNT)
				.into()
			)
		}
	}

	fn view_tasks<'a>(tasks: &'a HashMap<ProjectId, Vec<TaskId>>, app: &'a ProjectTrackerApp) -> Element<'a, Message> {
		Column::with_children(
			tasks.iter()
				.map(|(project_id, tasks)| {
					if let Some(project) = app.database.as_ref().and_then(|db| db.get_project(project_id)) {
						let list = Column::with_children(
							tasks.iter()
								.map(|task_id| {
									if let Some((task, task_type)) = project.get_task_and_type(task_id) {
										let stopwatch_label = match &app.content_page.stopwatch_page {
											StopwatchPage::StopTaskTime {
												project_id: timed_project_id,
												task_id: timed_task_id,
												clock,
												..
											} => {
												if *timed_project_id == *project_id && *timed_task_id == *task_id {
													Some(clock.label())
												} else {
													None
												}
											},
											_ => None,
										};
										task_widget(
											task,
											*task_id,
											task_type,
											*project_id,
											project,
											false,
											true,
											false,
											stopwatch_label,
											false
										)
									}
									else {
										text("<invalid task id>").into()
									}
								})
						);

						column![
							open_project_button(*project_id, &project.name, project.color.into()),
							list.padding(Padding::default().left(PADDING_AMOUNT)),
						]
						.into()
					}
					else {
						Element::new(text("<invalid project id>"))
					}
				})
		)
		.spacing(SPACING_AMOUNT)
		.padding(Padding::default().left(PADDING_AMOUNT))
		.into()
	}
}