use iced::{alignment::Horizontal, theme, widget::{button, column, container, row, text}, Element, Length, Padding};
use iced_aw::ContextMenu;
use crate::{project_tracker::UiMessage, styles::{ContextMenuContainerStyle, LIGHT_GREY, SMALL_PADDING_AMOUNT}};
use crate::components::{completion_bar, delete_project_button};
use crate::styles::{ProjectPreviewButtonStyle, SMALL_TEXT_SIZE, LARGE_TEXT_SIZE, SMALL_SPACING_AMOUNT, SMALL_HORIZONTAL_PADDING};
use crate::project::Project;

pub fn project_preview(project: &Project, selected: bool) -> Element<UiMessage> {
	custom_project_preview(
		Some(&project.name),
		project.get_completion_percentage(),
		project.get_tasks_done(),
		project.tasks.len(),
		text(&project.name)
			.size(LARGE_TEXT_SIZE)
			.into(),
		selected
	)
}

pub fn custom_project_preview<'a>(project_name: Option<&'a String>, project_completion_percentage: f32, tasks_done: usize, task_len: usize, inner_text_element: Element<'a, UiMessage>, selected: bool) -> Element<'a, UiMessage> {
	let inner = column![
		row![
			inner_text_element,
			container(
				text(format!("({}/{})", tasks_done, task_len))
					.style(theme::Text::Color(LIGHT_GREY))
					.size(SMALL_TEXT_SIZE)
			)
			.width(if project_name.is_some() { Length::Fill } else { Length::Shrink })
			.align_x(Horizontal::Right),
		]
		.width(Length::Fill)
		.spacing(SMALL_SPACING_AMOUNT),

		completion_bar(project_completion_percentage)
	]
	.padding(SMALL_HORIZONTAL_PADDING);

	let underlay = button(inner)
		.width(Length::Fill)
		.on_press_maybe(project_name.map(|project_name| UiMessage::SelectProject(project_name.clone())))
		.style(theme::Button::custom(ProjectPreviewButtonStyle{ selected }));

	if let Some(project_name) = project_name {
		let context_overlay = || {
			container(
				column![
					delete_project_button(project_name.clone()),
				]
			)
			.padding(Padding::new(SMALL_PADDING_AMOUNT))
			.style(theme::Container::Custom(Box::new(ContextMenuContainerStyle)))
			.into()
		};
	
		ContextMenu::new(underlay, context_overlay)
			.into()
	}
	else {
		container(
			underlay
		)
		.padding(Padding::new(SMALL_PADDING_AMOUNT))
		.style(theme::Container::Custom(Box::new(ContextMenuContainerStyle)))
		.into()
	}
}