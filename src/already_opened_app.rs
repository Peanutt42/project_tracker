use crate::{
	styles::{
		dangerous_button_style, ProjectTrackerTheme, FIRA_SANS_FONT, FIRA_SANS_FONT_BYTES,
		PADDING_AMOUNT,
	},
	theme_mode::is_system_theme_dark,
};
#[cfg(target_os = "linux")]
use iced::window::settings::PlatformSpecific;
use iced::{
	alignment::Horizontal,
	keyboard::{key, on_key_press, Key},
	widget::{button, column, container, text, Space},
	window::{self, icon},
	Element,
	Length::Fill,
	Result, Size, Subscription, Task, Theme,
};
use iced_fonts::REQUIRED_FONT_BYTES;

#[derive(Debug, Clone)]
enum Message {
	OkPressed,
}

struct AlreadyOpenedApp {
	system_theme_dark: bool,
}

impl AlreadyOpenedApp {
	fn new() -> Self {
		Self {
			system_theme_dark: is_system_theme_dark(),
		}
	}

	fn update(&mut self, message: Message) -> Task<Message> {
		match message {
			Message::OkPressed => window::get_latest().and_then(window::close),
		}
	}

	fn theme(&self) -> Theme {
		if self.system_theme_dark {
			ProjectTrackerTheme::Dark.get_theme().clone()
		} else {
			ProjectTrackerTheme::Light.get_theme().clone()
		}
	}

	fn subscription(&self) -> Subscription<Message> {
		on_key_press(|key, _modifiers| match key {
			Key::Named(key::Named::Enter) | Key::Named(key::Named::Escape) => {
				Some(Message::OkPressed)
			}
			_ => None,
		})
	}

	fn view(&self) -> Element<Message> {
		column![
			text("Project Tracker is already opened!")
				.width(Fill)
				.align_x(Horizontal::Center),
			Space::new(0.0, Fill),
			container(
				button(text("Ok").align_x(Horizontal::Center).width(Fill))
					.width(100)
					.style(dangerous_button_style)
					.on_press(Message::OkPressed)
			)
			.width(Fill)
			.align_x(Horizontal::Right)
		]
		.padding(PADDING_AMOUNT)
		.into()
	}
}

impl Default for AlreadyOpenedApp {
	fn default() -> Self {
		Self::new()
	}
}

pub fn run_already_opened_application() -> Result {
	iced::application(
		"Project Tracker already opened",
		AlreadyOpenedApp::update,
		AlreadyOpenedApp::view,
	)
	.theme(AlreadyOpenedApp::theme)
	.subscription(AlreadyOpenedApp::subscription)
	.font(REQUIRED_FONT_BYTES)
	.font(FIRA_SANS_FONT_BYTES)
	.default_font(FIRA_SANS_FONT)
	.antialiasing(true)
	.window(window::Settings {
		icon: icon::from_file_data(
			include_bytes!("../assets/icon.png"),
			Some(image::ImageFormat::Png),
		)
		.ok(),
		size: Size::new(300.0, 100.0),
		resizable: false,
		#[cfg(target_os = "linux")]
		platform_specific: PlatformSpecific {
			application_id: "project_tracker".to_string(),
			..Default::default()
		},
		..Default::default()
	})
	.run()
}
