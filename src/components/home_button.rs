use iced::{theme, widget::{button, svg, Button}, Border, Vector, Color, Shadow, Theme};
use crate::project_tracker::{UiMessage, ProjectTrackerPage};

use super::create_new_project::CreateNewProjectModal;

pub fn home_button() -> Button<'static, UiMessage>{
	let home_svg = svg::Handle::from_path(format!("{}/assets/home.svg", env!("CARGO_MANIFEST_DIR")));

	button(
		svg(home_svg)
			.width(24)
			.height(24)
			.style(theme::Svg::Custom(Box::new(HomeSvgStyle)))
	)
	.on_press(UiMessage::SwitchPage(ProjectTrackerPage::StartPage{ create_new_project_modal: CreateNewProjectModal::new() }))
	.style(theme::Button::Secondary)
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
