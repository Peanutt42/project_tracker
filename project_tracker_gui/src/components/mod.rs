mod completion_bar;
pub use completion_bar::completion_bar;

mod buttons;
pub use buttons::{
	add_due_date_button, cancel_create_new_task_tag_button, cancel_create_project_button,
	cancel_search_tasks_button, clear_synchronization_filepath_button,
	clear_task_due_date_button, clear_task_needed_time_button, color_palette_item_button,
	complete_task_timer_button, confirm_cancel_button, confirm_ok_button, copy_to_clipboard_button,
	create_new_project_button, open_create_task_modal_button, create_new_task_modal_button, close_create_new_task_modal_button, create_new_task_tags_button,
	dangerous_button, date_formatting_button, delete_all_done_tasks_button,
	project_context_menu_button, edit_task_button, finish_editing_task_button, delete_task_button, delete_task_tag_button, edit_color_palette_button,
	edit_due_date_button, edit_project_name_button, export_database_button, import_database_button,
	import_google_tasks_button,
	pause_timer_button, reimport_source_code_todos_button, resume_timer_button,
	search_tasks_button, select_synchronization_filepath_button, settings_button,
	settings_tab_button, show_done_tasks_button, show_source_code_todos_button,
	start_task_timer_button, start_timer_button, stop_timer_button, stopwatch_button,
	sync_database_button, task_tag_button, theme_mode_button, toggle_sidebar_button,
	open_link_button, ICON_BUTTON_WIDTH,
};

mod task_list;
pub use task_list::task_list;

mod task_widget;
pub use task_widget::{task_widget, EDIT_DUE_DATE_TEXT_INPUT_ID, EDIT_NEEDED_TIME_TEXT_INPUT_ID};

mod task_tags_widget;
pub use task_tags_widget::task_tags_buttons;

mod duration_widget;
pub use duration_widget::{duration_text, duration_widget};

mod date_widget;
pub use date_widget::{date_text, days_left_widget};

mod loading_screen;
pub use loading_screen::loading_screen;

mod project_preview;
pub use project_preview::{custom_project_preview, project_color_block, project_preview};

mod seperator;
pub use seperator::{horizontal_seperator, horizontal_seperator_padded, vertical_seperator};

mod file_location;
pub use file_location::{file_location, filepath_widget};

mod color_palette;
pub use color_palette::{color_palette, COLOR_PALETTE_BLACK, COLOR_PALETTE_WHITE};

mod unfocusable;
pub use unfocusable::unfocusable;

mod scrollable;
pub use scrollable::{horizontal_scrollable, vertical_scrollable, HORIZONTAL_SCROLLABLE_PADDING};

mod dropzone;
pub use dropzone::in_between_dropzone;

mod stopwatch_clock;
pub use stopwatch_clock::StopwatchClock;

mod animation;
pub use animation::ScalarAnimation;
