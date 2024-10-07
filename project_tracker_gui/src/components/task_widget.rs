use crate::{
	core::{
		DatabaseMessage, DateFormatting, OrderedHashMap, ProjectId, Task, TaskId, TaskTag,
		TaskTagId, TaskType, TASK_TAG_QUAD_HEIGHT,
	},
	pages::{EditTaskState, ProjectPageMessage, SidebarPageMessage},
	components::{
		add_due_date_button, clear_task_due_date_button, clear_task_needed_time_button,
		days_left_widget, delete_task_button, duration_text, duration_widget, edit_due_date_button,
		in_between_dropzone, start_task_timer_button, task_tags_buttons, unfocusable,
		edit_task_button, finish_editing_task_button, ICON_BUTTON_WIDTH
	},
	icons::{icon_to_text, Bootstrap},
	project_tracker::Message,
	styles::{
		checkbox_style, secondary_button_style_default, secondary_button_style_only_round_bottom,
		shadow_container_style, task_background_container_style, task_button_style,
		text_editor_style, text_input_style, SMALL_HORIZONTAL_PADDING, SMALL_PADDING_AMOUNT,
		SMALL_SPACING_AMOUNT, SMALL_TEXT_SIZE, SPACING_AMOUNT, TINY_SPACING_AMOUNT, DARK_THEME,
		link_color, PADDING_AMOUNT
	},
};
use iced::keyboard::{self, key};
use iced::widget::{hover, markdown, Space};
use iced::widget::text_editor::{Binding, KeyPress, Motion, Status};
use iced::{
	alignment::Vertical,
	widget::{
		button, checkbox, column, container, container::Id, row, text, text_editor, text_input,
		Column, Row,
	},
	Alignment, Element,
	Length::{self, Fill},
	Padding, Theme,
};
use iced_aw::{date_picker, date_picker::Date};
use iced_drop::droppable;
use once_cell::sync::Lazy;
use std::{borrow::Cow, str::FromStr, time::Duration};

pub static EDIT_NEEDED_TIME_TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);
pub static EDIT_DUE_DATE_TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

#[allow(clippy::too_many_arguments)]
pub fn task_widget<'a>(
	task: &'a Task,
	task_id: TaskId,
	task_type: TaskType,
	project_id: ProjectId,
	task_tags: &'a OrderedHashMap<TaskTagId, TaskTag>,
	edit_task_state: Option<&'a EditTaskState>,
	dragging: bool,
	just_minimal_dragging: bool,
	highlight: bool,
	stopwatch_label: Option<&'a String>,
	date_formatting: DateFormatting,
	is_theme_dark: bool
) -> Element<'a, Message> {
	if let Some(edit_task_state) = edit_task_state {
		edit_task_widget_view(
			task,
			project_id,
			task_id,
			task_tags,
			edit_task_state,
			date_formatting,
		)
	} else {
		task_widget_view(
			task,
			task_type,
			project_id,
			task_id,
			task_tags,
			dragging,
			just_minimal_dragging,
			highlight,
			stopwatch_label,
			is_theme_dark
		)
	}
}

fn edit_task_widget_view<'a>(
	task: &'a Task,
	project_id: ProjectId,
	task_id: TaskId,
	task_tags: &'a OrderedHashMap<TaskTagId, TaskTag>,
	edit_task_state: &'a EditTaskState,
	date_formatting: DateFormatting,
) -> Element<'a, Message> {
	let edit_task_name_text_editor: Element<'a, Message> = {
		let round_top_left = task.needed_time_minutes.is_none()
			&& task_tags
				.get_key_at_order(0)
				.map(|tag_id| !task.tags.contains(tag_id))
				.unwrap_or(true);

		unfocusable(
			container(
				text_editor(&edit_task_state.new_name)
					.on_action(move |action| ProjectPageMessage::TaskNameAction(action).into())
					.style(move |t, s| {
						text_editor_style(t, s, round_top_left, true, false, true)
					})
					.key_binding(|key_press| {
						let KeyPress {
							key,
							modifiers,
							status,
							..
						} = &key_press;

						if *status != Status::Focused {
							return None;
						}

						match key {
							keyboard::Key::Named(key::Named::Delete) => Some(
								if modifiers.command() {
									Binding::<Message>::Sequence(vec![
										Binding::Select(Motion::WordRight),
										Binding::Delete,
									])
								}
								else {
									Binding::Delete
								}
							),
							keyboard::Key::Named(key::Named::Backspace) if modifiers.command() => Some(
								Binding::<Message>::Sequence(vec![
									Binding::<Message>::Select(Motion::WordLeft),
									Binding::Backspace,
								])
							),
							_ => Binding::<Message>::from_key_press(key_press)
						}
					})
			)
			.style(shadow_container_style),
			ProjectPageMessage::FinishEditingTask.into(),
		)
		.into()
	};

	column![
		task_tags_buttons(task_tags, &task.tags, |tag_id| {
			ProjectPageMessage::ToggleTaskTag(tag_id).into()
		}),
		row![
			Row::new()
				.push_maybe(task.needed_time_minutes.as_ref().map(|_| {
					start_task_timer_button(
						project_id,
						task_id,
						task_tags
							.iter()
							.next()
							.map(|(tag_id, _tag)| !task.tags.contains(&tag_id))
							.unwrap_or(true),
					)
				}))
				.push(column![
					edit_task_name_text_editor,
					row![
						if let Some(new_task_needed_minutes) =
							&edit_task_state.new_needed_time_minutes
						{
							let stop_editing_task_message: Message =
								ProjectPageMessage::FinishEditingTask.into();

							let edit_needed_time_element = unfocusable(
								text_input(
									"mins",
									&match new_task_needed_minutes {
										Some(needed_time_minutes) => {
											format!("{needed_time_minutes}")
										}
										None => String::new(),
									},
								)
								.id(EDIT_NEEDED_TIME_TEXT_INPUT_ID.clone())
								.width(Length::Fixed(50.0))
								.on_input(move |input| {
									let new_needed_time_minutes = match usize::from_str(&input) {
										Ok(new_needed_time_minutes) => {
											Some(Some(new_needed_time_minutes))
										}
										Err(_) => {
											if input.is_empty() {
												Some(None)
											} else {
												None
											}
										}
									};
									match new_needed_time_minutes {
										Some(new_needed_time_minutes) => {
											ProjectPageMessage::ChangeNewTaskNeededTimeInput(
												new_needed_time_minutes,
											)
											.into()
										}
										None => ProjectPageMessage::InvalidNeededTimeInput.into(),
									}
								})
								.on_submit(ProjectPageMessage::ChangeTaskNeededTime.into())
								.style(move |t, s| {
									text_input_style(t, s, false, false, false, true)
								}),
								stop_editing_task_message,
							);

							row![edit_needed_time_element, clear_task_needed_time_button(),].into()
						} else {
							Element::new(
								button(
									row![
										icon_to_text(Bootstrap::Stopwatch),
										if let Some(needed_duration_minutes) =
											&task.needed_time_minutes
										{
											duration_text(Cow::Owned(Duration::from_secs(
												*needed_duration_minutes as u64 * 60,
											)))
										} else {
											text("Add needed time")
										}
									]
									.spacing(SMALL_SPACING_AMOUNT),
								)
								.padding(SMALL_HORIZONTAL_PADDING)
								.on_press(ProjectPageMessage::EditTaskNeededTime.into())
								.style(secondary_button_style_only_round_bottom),
							)
						},
						if edit_task_state.edit_due_date {
							Element::new(date_picker(
								edit_task_state.edit_due_date,
								task.due_date.unwrap_or(Date::today().into()),
								text("Edit due date"),
								ProjectPageMessage::StopEditingTaskDueDate.into(),
								move |date| {
									ProjectPageMessage::ChangeTaskDueDate(date.into()).into()
								},
							))
						} else if let Some(due_date) = &task.due_date {
							row![
								edit_due_date_button(due_date, date_formatting),
								clear_task_due_date_button(),
							]
							.into()
						} else {
							Element::new(add_due_date_button())
						},
					]
					.spacing(SPACING_AMOUNT),
				])
				.align_y(Alignment::Start),

			finish_editing_task_button(),

			delete_task_button(project_id, task_id)
		]
		.align_y(Alignment::Start)
	]
	.padding(Padding {
		top: SPACING_AMOUNT as f32,
		..Padding::ZERO
	})
	.into()
}

#[allow(clippy::too_many_arguments)]
fn task_widget_view<'a>(
	task: &'a Task,
	task_type: TaskType,
	project_id: ProjectId,
	task_id: TaskId,
	task_tags: &'a OrderedHashMap<TaskTagId, TaskTag>,
	dragging: bool,
	just_minimal_dragging: bool,
	highlight: bool,
	stopwatch_label: Option<&'a String>,
	is_theme_dark: bool,
) -> Element<'a, Message> {
	let tags_element = Row::with_children(
		task_tags
			.iter()
			.filter(|(tag_id, _tag)| task.tags.contains(tag_id))
			.map(|(_tag_id, tag)| tag.view()),
	)
	.spacing(TINY_SPACING_AMOUNT);

	// TODO: how should markdown look on done tasks?
	let inner_text_element: Element<'a, Message> = markdown(
		task.markdown_items(),
		markdown::Settings::default(),
		markdown::Style {
			link_color: link_color(is_theme_dark),
			..markdown::Style::from_palette(DARK_THEME.palette())
		}
	)
	.map(|markdown_url| Message::OpenUrl(markdown_url.to_string()));

	let on_hover_view: Element<'a, Message> = row![
		if dragging {
			Space::new(0.0, 0.0).into()
		}
		else {
			Element::new(
				container(icon_to_text(Bootstrap::GripVertical))
					.padding(Padding {
						top: if task.tags.is_empty() {
							0.0
						} else {
							TASK_TAG_QUAD_HEIGHT + TINY_SPACING_AMOUNT * 1.5
						},
						..Padding::ZERO
					})
					.center_y(Fill)
			)
		},

		Space::new(Fill, 0.0),

		if dragging {
			Space::new(0.0, 0.0).into()
		}
		else {
			Element::new(edit_task_button(task_id))
		}
	]
	.into();

	let grip_icon_dummy: Element<Message> = if dragging {
		container(
			icon_to_text(Bootstrap::GripVertical)
		)
		.padding(Padding {
			top: if task.tags.is_empty() {
				0.0
			} else {
				TASK_TAG_QUAD_HEIGHT + TINY_SPACING_AMOUNT * 1.5
			},
			..Padding::ZERO
		})

		.into()
	}
	else {
		Space::new(PADDING_AMOUNT, 0.0).into()
	};

	let edit_task_button_dummy: Element<Message> = if dragging {
		edit_task_button(task_id).into()
	}
	else {
		Space::new(ICON_BUTTON_WIDTH, 0.0).into()
	};

	let inner: Element<'a, Message> = row![
		grip_icon_dummy,
		row![
			container(
				checkbox("", matches!(task_type, TaskType::Done))
					.on_toggle(move |checked| {
						if checked {
							DatabaseMessage::SetTaskDone {
								project_id,
								task_id,
							}
							.into()
						} else {
							DatabaseMessage::SetTaskTodo {
								project_id,
								task_id,
							}
							.into()
						}
					})
					.style(checkbox_style)
			)
			.padding(Padding {
				top: if task.tags.is_empty() {
					0.0
				} else {
					TASK_TAG_QUAD_HEIGHT + TINY_SPACING_AMOUNT
				},
				..Padding::ZERO
			}),
			Column::new()
				.push_maybe(if task.tags.is_empty() {
					None
				} else {
					Some(tags_element)
				})
				.push(inner_text_element)
				.spacing(TINY_SPACING_AMOUNT),
			Column::new().push_maybe(
				if task.needed_time_minutes.is_some() || task.due_date.is_some() {
					Some(
						Column::new()
							.push_maybe(task.needed_time_minutes.map(|duration_minutes| {
								duration_widget(Cow::Owned(Duration::from_secs(
									duration_minutes as u64 * 60,
								)))
							}))
							.push_maybe(
								task.due_date
									.as_ref()
									.map(|due_date| days_left_widget(*due_date)),
							)
							.push_maybe(stopwatch_label.map(|label| -> Element<Message> {
								button(
									row![
										icon_to_text(Bootstrap::Stopwatch).size(SMALL_TEXT_SIZE),
										text(label).style(|theme: &Theme| text::Style {
											color: Some(theme.extended_palette().danger.base.color)
										})
									]
									.align_y(Vertical::Center)
									.spacing(TINY_SPACING_AMOUNT),
								)
								.padding(SMALL_HORIZONTAL_PADDING)
								.style(secondary_button_style_default)
								.on_press(Message::OpenStopwatch)
								.into()
							}))
							.spacing(TINY_SPACING_AMOUNT)
							.align_x(Alignment::End)
							.into(),
					) as Option<Element<Message>>
				} else {
					None
				}
			),
			edit_task_button_dummy
		]
		.width(Fill)
		.align_y(Alignment::Start)
	]
	.align_y(Vertical::Center)
	.into();

	column![
		in_between_dropzone(
			if matches!(task_type, TaskType::Todo) {
				task.dropzone_id.clone()
			} else {
				Id::unique()
			},
			highlight
		),
		hover(
			droppable(
				container(inner)
					.padding(Padding::new(SMALL_PADDING_AMOUNT))
					.style(move |t| task_background_container_style(
						t,
						dragging && !just_minimal_dragging
					))
			)
			.on_drop(move |point, rect| SidebarPageMessage::DropTask {
				project_id,
				task_id,
				point,
				rect
			}
			.into())
			.on_drag(move |point, rect| Message::DragTask {
				project_id,
				task_id,
				task_is_todo: matches!(task_type, TaskType::Todo),
				point,
				rect
			})
			.on_cancel(Message::CancelDragTask)
			.drag_overlay(!just_minimal_dragging)
			.drag_hide(!just_minimal_dragging)
			.style(task_button_style),

			on_hover_view
		)
	]
	.into()
}
