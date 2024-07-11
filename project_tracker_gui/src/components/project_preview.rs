use iced::{alignment::Horizontal, theme, widget::{button, column, container, row, text, text_input}, Alignment, Border, Color, Element, Length, Padding};
use iced_aw::{quad::Quad, widgets::InnerBounds};
use once_cell::sync::Lazy;
use crate::{project_tracker::UiMessage, styles::SMALL_PADDING_AMOUNT};
use crate::components::{completion_bar, cancel_create_project_button};
use crate::styles::{ProjectPreviewButtonStyle, SMALL_TEXT_SIZE, LARGE_TEXT_SIZE, LIGHT_GREY, TINY_SPACING_AMOUNT, SMALL_SPACING_AMOUNT};
use crate::core::{Project, ProjectId};

pub static EDIT_PROJECT_NAME_TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

pub fn project_color_block(color: Color, height: f32) -> Element<'static, UiMessage> {
	Quad {
		width: Length::Fixed(5.0),
		height: Length::Fixed(height),
		inner_bounds: InnerBounds::Ratio(1.0, 1.0),
		quad_color: color.into(),
		quad_border: Border::with_radius(f32::MAX),
		..Default::default()
	}
	.into()
}

pub fn project_preview(project: &Project, project_id: ProjectId, selected: bool) -> Element<UiMessage> {
	let inner_text_element = text(&project.name).size(LARGE_TEXT_SIZE).into();

	custom_project_preview(
		Some(project_id),
		project.color.into(),
		project.get_completion_percentage(),
		project.get_tasks_done(),
		project.tasks.len(),
		inner_text_element,
		selected
	)
}

#[allow(clippy::too_many_arguments)]
pub fn custom_project_preview(project_id: Option<ProjectId>, project_color: Color, project_completion_percentage: f32, tasks_done: usize, task_len: usize, inner_text_element: Element<UiMessage>, selected: bool) -> Element<UiMessage> {
	let inner = row![
		project_color_block(project_color, 35.0),

		column![
			row![
				inner_text_element,
				container(
					text(format!("({}/{})", tasks_done, task_len))
						.style(theme::Text::Color(LIGHT_GREY))
						.size(SMALL_TEXT_SIZE)
				)
				.width(if project_id.is_some() { Length::Fill } else { Length::Shrink })
				.align_x(Horizontal::Right),
			]
			.width(Length::Fill)
			.spacing(SMALL_SPACING_AMOUNT),

			completion_bar(project_completion_percentage)
		]
		.spacing(TINY_SPACING_AMOUNT)
	]
	.align_items(Alignment::Center)
	.spacing(TINY_SPACING_AMOUNT)
	.padding(Padding{ right: SMALL_PADDING_AMOUNT, ..Padding::ZERO });

	let underlay =
		container(
				button(inner)
					.width(Length::Fill)
					.on_press(UiMessage::SelectProject(project_id))
					.style(theme::Button::custom(ProjectPreviewButtonStyle{ selected }))
		)
		.width(Length::Fill);

	if project_id.is_some() {
		underlay.into()
	}
	else {
		row![
			underlay,
			cancel_create_project_button()
		]
		.align_items(Alignment::Center)
		.spacing(SMALL_SPACING_AMOUNT)
		.width(Length::Fill)
   		.into()
	}
}
