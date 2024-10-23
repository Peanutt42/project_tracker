use iced::{Task, Element, window, widget::text};

#[derive(Debug, Default)]
struct Dashboard {

}

#[derive(Debug, Clone)]
enum Message {

}

impl Dashboard {
	fn new() -> (Self, Task<Message>) {
		(
			Self {},
			window::get_latest()
				.and_then(|id| window::change_mode(id, window::Mode::Fullscreen))
		)
	}

	fn update(&mut self, _message: Message) -> Task<Message> {
		Task::none()
	}

	fn view(&self) -> Element<Message> {
		text("Server Dashboard WIP")
			.size(40)
			.into()
	}
}

pub fn run_dashboard() {
	println!("Starting Dashboard GUI");

	let result = iced::application(
		"Project Tracker Server Dashboard",
		Dashboard::update,
		Dashboard::view
	)
	.run_with(Dashboard::new);

	if let Err(e) = result {
		eprintln!("failed to run dashboard app: {e}");
	}
}