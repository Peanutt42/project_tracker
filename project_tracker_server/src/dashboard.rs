use project_tracker_server::ServerEvent;
use iced::{Task, Element, Subscription, Theme, window, widget::text, stream};
use iced::futures::{SinkExt, Stream};
use std::path::PathBuf;
use crate::server::{self, create_server};

#[derive(Debug)]
struct Dashboard {
	database_filepath: PathBuf,
	server_event_str: String,
}

#[derive(Debug, Clone)]
pub enum Message {
	ServerEvent(ServerEvent),
}

impl Dashboard {
	fn new(database_filepath: PathBuf) -> (Self, Task<Message>) {
		(
			Self { database_filepath, server_event_str: String::new() },
			window::get_latest()
				.and_then(|id| window::change_mode(id, window::Mode::Fullscreen))
		)
	}

	fn update(&mut self, message: Message) -> Task<Message> {
		match message {
			Message::ServerEvent(server_event) => self.server_event_str = format!("{server_event:?}"),
		}
		Task::none()
	}

	fn view(&self) -> Element<Message> {
		text(format!("Server Dashboard WIP - {}", self.server_event_str))
			.size(40)
			.into()
	}

	fn subscription(&self) -> Subscription<Message> {
		Subscription::run_with_id("ProjectTrackerServer", server_subscription(self.database_filepath.clone()))
			.map(Message::ServerEvent)
	}

	fn theme(&self) -> Theme {
		Theme::Dark
	}
}

pub fn run_dashboard(filepath: PathBuf) {
	println!("Starting Dashboard GUI");

	let result = iced::application(
		"Project Tracker Server Dashboard",
		Dashboard::update,
		Dashboard::view
	)
	.subscription(Dashboard::subscription)
	.theme(Dashboard::theme)
	.run_with(|| Dashboard::new(filepath));

	if let Err(e) = result {
		eprintln!("failed to run dashboard app: {e}");
	}
}

fn server_subscription(filepath: PathBuf) -> impl Stream<Item = ServerEvent> {
	stream::channel(1, |mut gui_sender| async move {
		let server = create_server().await;
		loop {
			match server.accept().await {
				Ok((stream, _addr)) => {
					let filepath_clone = filepath.clone();
					let mut gui_sender_clone = gui_sender.clone();

					tokio::spawn(async move {
						if let Some(server_event) = server::handle_client(stream, filepath_clone).await {
							if gui_sender_clone.send(server_event).await.is_err() {
								eprintln!("failed to send server event to dashboard!");
							}
						}
					});
				},
				Err(e) => {
					let error_msg = format!("failed to establish a connection: {e}");
					eprintln!("{error_msg}");
					if gui_sender.send(ServerEvent::Error(error_msg)).await.is_err() {
						eprintln!("failed to send server event to dashboard!");
					}
				}
			}
		}
	})
}