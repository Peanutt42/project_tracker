use iced::{theme, widget::{button, column, container, text}, Border, Color, Element, Shadow, Theme, Vector};
use crate::{project::Project, project_tracker::UiMessage};
use crate::components::completion_bar;

pub fn project_preview(project: &Project) -> Element<UiMessage> {
	button(
		container(
			column![
				// TODO: icon
				text(&project.name)
					.size(35)
					.horizontal_alignment(iced::alignment::Horizontal::Center),
				completion_bar(project.get_completion_percentage())
			]
			.align_items(iced::Alignment::Center)
			.spacing(5.0)
		)
		.padding(20)
	)
	.on_press(UiMessage::GotoProjectPage(project.name.clone()))
	.style(theme::Button::Custom(Box::new(ProjectPreviewButtonStyle)))
	.into()
}

struct ProjectPreviewButtonStyle;

impl button::StyleSheet for ProjectPreviewButtonStyle {
	type Style = Theme;

	fn active(&self, style: &Self::Style) -> button::Appearance {
		button::Appearance {
			border: Border::with_radius(10.0),
			shadow: Shadow {
				color: Color::BLACK,
				offset: Vector::default(),
				blur_radius: 20.0,
			},
			text_color: style.palette().text,
			..Default::default()
		}
	}

	fn hovered(&self, style: &Self::Style) -> button::Appearance {
		button::Appearance {
			border: Border::with_radius(10.0),
			shadow: Shadow {
				color: Color::BLACK,
				offset: Vector::default(),
				blur_radius: 30.0,
			},
			text_color: style.palette().text,
			..Default::default()
		}
	}

	fn pressed(&self, style: &Self::Style) -> button::Appearance {
		button::Appearance {
			border: Border::with_radius(10.0),
			shadow: Shadow {
				color: Color::BLACK,
				offset: Vector::default(),
				blur_radius: 35.0,
			},
			text_color: style.palette().text,
			..Default::default()
		}
	}
}
