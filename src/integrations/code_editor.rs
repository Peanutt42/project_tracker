use std::{process::Command, sync::LazyLock};
use iced::{Element, widget::{image, image::Handle}, advanced::image::Bytes};
use serde::{Deserialize, Serialize};

use crate::{components::ICON_FONT_SIZE, icons::{icon_to_text, Bootstrap}, project_tracker::Message};

static VSCODE_ICON_IMAGE_HANDLE: LazyLock<Handle> = LazyLock::new(|| Handle::from_bytes(Bytes::from_static(include_bytes!("../../assets/vscode_icon.ico"))));
static ZED_ICON_IMAGE_HANDLE: LazyLock<Handle> = LazyLock::new(|| Handle::from_bytes(Bytes::from_static(include_bytes!("../../assets/zed_icon.png"))));

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CodeEditor {
	VSCode,
	Zed,
	Custom {
		name: String,
		command: String,
	}
}

impl CodeEditor {
	pub fn label(&self) -> &str {
		match self {
			Self::VSCode => "VS Code",
			Self::Zed => "Zed",
			Self::Custom { .. } => "Custom",
		}
	}

	pub fn name(&self) -> &str {
		match self {
			Self::VSCode => "VS Code",
			Self::Zed => "Zed",
			Self::Custom { name, .. } => name
		}
	}

	pub fn icon(&self) -> Element<'static, Message> {
		match self {
			Self::VSCode => image(VSCODE_ICON_IMAGE_HANDLE.clone())
				.height(ICON_FONT_SIZE)
				.into(),
			Self::Zed => image(ZED_ICON_IMAGE_HANDLE.clone())
				.height(ICON_FONT_SIZE)
				.into(),
			Self::Custom { .. } => icon_to_text(Bootstrap::CodeSlash)
				.size(ICON_FONT_SIZE)
				.into(),
		}
	}

	pub fn generate_command(&self, file_location: &str) -> Command {
		match self {
			Self::VSCode => {
				let mut command;
				let mut args = Vec::new();
				// if vscode is installed natively -> run code
				// if installed with flatpak -> run flatpak
				if which::which("code").is_ok() {
					command = Command::new("code");
				} else {
					command = Command::new("flatpak");
					args.push("run");
					args.push("com.visualstudio.code");
				}
				args.push("--goto");
				args.push(file_location);
				command.args(args);
				command
			},
			Self::Zed => {
				let mut command = Command::new("zed");
				command.args([file_location]);
				command
			},
			Self::Custom { command, .. } => {
				let mut split_command = command.split(' ');
				let program = split_command.next().unwrap_or("");
				let mut command = Command::new(program);
				let mut args: Vec<&str> = split_command.collect();
				args.push(file_location);
				command.args(args);
				command
			},
		}
	}
}