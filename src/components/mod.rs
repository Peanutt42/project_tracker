mod completion_bar;
pub use completion_bar::completion_bar;

mod buttons;
pub use buttons::{
	cancel_create_new_task_tag_button, cancel_create_project_button, cancel_search_tasks_button,
	close_create_new_task_modal_button, code_editor_dropdown_button, color_palette_item_button,
	complete_task_timer_button, confirm_cancel_button, confirm_ok_button, copy_to_clipboard_button,
	create_empty_database_button, create_new_project_button, create_new_task_modal_button,
	create_new_task_tags_button, dangerous_button, date_formatting_button,
	delete_all_done_tasks_button, delete_task_button, delete_task_tag_button, due_date_button,
	edit_color_palette_button, edit_needed_time_button, error_msg_ok_button,
	export_as_json_database_button, export_database_button, force_close_anyways_button,
	hide_password_button, import_database_button, import_google_tasks_button,
	import_json_database_button, open_create_task_modal_button, open_folder_location_button,
	open_in_code_editor_button, open_project_button, open_task_by_name_link_button,
	overview_button, overview_time_section_button, pause_timer_button, project_context_menu_button,
	reimport_source_code_todos_button, resume_timer_button, retry_loading_database_button,
	retry_synchronization_button, search_tasks_button, settings_button, settings_tab_button,
	show_done_tasks_button, show_error_popup_button, show_password_button,
	show_source_code_todos_button, sort_dropdown_button, start_task_timer_button,
	stop_timer_button, stopwatch_button, synchronization_settings_button, take_break_button,
	task_tag_button, task_tag_name_button, theme_mode_button, toggle_sidebar_button,
	toggle_view_edit_task_description_button, track_time_button, ICON_BUTTON_WIDTH, ICON_FONT_SIZE,
};

mod task_list;
pub use task_list::task_list;

mod task_widget;
pub use task_widget::task_widget;

mod task_description;
pub use task_description::{
	generate_task_description_markdown, task_description, task_description_editor,
};

mod duration_widget;
pub use duration_widget::{
	duration_str, duration_text, duration_to_minutes, duration_widget, parse_duration_from_str,
	round_duration_to_seconds,
};

mod date_widget;
pub use date_widget::{date_text, days_left_widget};

mod loading_screen;
pub use loading_screen::{loading_screen, LARGE_LOADING_SPINNER_SIZE, SMALL_LOADING_SPINNER_SIZE};

mod project_preview;
pub use project_preview::{custom_project_preview, project_preview};

mod seperator;
pub use seperator::{horizontal_seperator, horizontal_seperator_padded, vertical_seperator};

mod file_location;
pub use file_location::{file_location, filepath_widget};

mod color_palette;
pub use color_palette::{color_palette, COLOR_PALETTE_BLACK, COLOR_PALETTE_WHITE};

mod unfocusable;
pub use unfocusable::unfocusable;

mod scrollable;
pub use scrollable::{
	horizontal_scrollable, vertical_scrollable, vertical_scrollable_no_padding,
	HORIZONTAL_SCROLLABLE_PADDING, SCROLLBAR_WIDTH,
};

mod dropzone;
pub use dropzone::in_between_dropzone;

mod stopwatch_clock;
pub use stopwatch_clock::StopwatchClock;

mod animation;
pub use animation::ScalarAnimation;

pub mod markdown;
