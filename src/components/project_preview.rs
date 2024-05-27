use iced::{alignment::Horizontal, theme, widget::{button, column, container, row, text}, Element, Length, Padding};
use iced_aw::ContextMenu;
use crate::{project_tracker::UiMessage, styles::{ContextMenuContainerStyle, LIGHT_GREY, SMALL_PADDING_AMOUNT}};
use crate::components::{completion_bar, delete_project_button};
use crate::styles::{ProjectPreviewButtonStyle, SMALL_TEXT_SIZE, LARGE_TEXT_SIZE, SMALL_SPACING_AMOUNT, SMALL_HORIZONTAL_PADDING};
use crate::project::Project;

pub fn project_preview(project: &Project, selected: bool) -> Element<UiMessage> {
	let inner = column![
		row![
			text(&project.name).size(LARGE_TEXT_SIZE),
			container(
				text(format!("({}/{})", project.get_tasks_done(), project.tasks.len()))
					.style(theme::Text::Color(LIGHT_GREY))
					.size(SMALL_TEXT_SIZE)
			)
			.width(Length::Fill)
			.align_x(Horizontal::Right),
		]
		.spacing(SMALL_SPACING_AMOUNT),
		completion_bar(project.get_completion_percentage())
	]
	.padding(SMALL_HORIZONTAL_PADDING);

	let underlay = button(inner)
		.width(Length::Fill)
		.on_press(UiMessage::SelectProject(project.name.clone()))
		.style(theme::Button::custom(ProjectPreviewButtonStyle{ selected }));

	let context_overlay = || {
		container(
			column![
				delete_project_button(project.name.clone()),
			]
		)
		.padding(Padding::new(SMALL_PADDING_AMOUNT))
		.style(theme::Container::Custom(Box::new(ContextMenuContainerStyle)))
		.into()
	};

	ContextMenu::new(underlay, context_overlay)
		.into()
}