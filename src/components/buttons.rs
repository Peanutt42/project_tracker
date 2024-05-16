use iced::{theme, widget::{button, svg, Button}, Border, Vector, Color, Shadow, Theme};
use crate::{
	project_tracker::UiMessage,
	components::{CreateNewProjectModalMessage, CreateNewTaskModalMessage},
	styles::GreenCircleButtonStyle,
};

pub fn home_button() -> Button<'static, UiMessage>{
	let home_svg = svg::Handle::from_path(format!("{}/assets/home.svg", env!("CARGO_MANIFEST_DIR")));

	button(
		svg(home_svg)
			.width(32)
			.height(32)
			.style(theme::Svg::Custom(Box::new(HomeSvgStyle)))
	)
	.on_press(UiMessage::GotoStartPage)
	.style(theme::Button::Secondary)
}

pub fn create_new_project_button() -> Button<'static, UiMessage> {
	let add_project_svg = svg::Handle::from_memory(include_bytes!("../../assets/add_project.svg"));

	button(
		svg(add_project_svg)
			.width(32)
			.height(32)
	)
	.on_press(CreateNewProjectModalMessage::Open.into())
	.style(iced::theme::Button::Custom(Box::new(GreenCircleButtonStyle)))
}

pub fn create_new_task_button() -> Button<'static, UiMessage> {
	let add_task_svg = svg::Handle::from_memory(include_bytes!("../../assets/add_task.svg"));

	button(
		svg(add_task_svg)
			.width(32)
			.height(32)
	)
	.on_press(CreateNewTaskModalMessage::Open.into())
	.style(theme::Button::Custom(Box::new(GreenCircleButtonStyle)))
}


struct HomeSvgStyle;

impl svg::StyleSheet for HomeSvgStyle {
	type Style = Theme;

	fn appearance(&self, style: &Self::Style) -> svg::Appearance {
		svg::Appearance {
			color: Some(
				if style.extended_palette().is_dark {
					Color::WHITE
				}
				else {
					Color::BLACK
				}
			),
		}
	}
}

struct HomeButtonStyle;

impl button::StyleSheet for HomeButtonStyle {
	type Style = Theme;

	fn active(&self, _style: &Self::Style) -> button::Appearance {
		button::Appearance {
			border: Border::with_radius(2.0),
			background: Some(iced::Background::Color(_style.palette().background)),
			shadow: Shadow {
				color: Color::BLACK,
				offset: Vector::default(),
				blur_radius: 10.0,
			},
			..Default::default()
		}
	}
}
