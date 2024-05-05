
#[macro_export]
macro_rules! include_icon {
	($file:literal) => {{
		let icon_bytes = include_bytes!($file);
		if let Ok(icon_image_reader) = image::io::Reader::new(std::io::Cursor::new(icon_bytes)).with_guessed_format() {
			if let Ok(icon_image) = icon_image_reader.decode() {
				iced::window::icon::from_rgba(icon_image.as_bytes().to_vec(), icon_image.width(), icon_image.height()).ok()
			}
			else {
				None
			}
		}
		else {
			None
		}
	}};
}
