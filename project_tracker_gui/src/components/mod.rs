mod completion_bar;
pub use completion_bar::completion_bar;

mod buttons;
pub use buttons::{create_new_project_button, create_new_task_button, delete_project_button, delete_task_button, delete_all_done_tasks_button, show_done_tasks_button, cancel_create_project_button, cancel_create_task_button, stopwatch_button, dangerous_button, theme_mode_button, settings_button, copy_to_clipboard_button, toggle_sidebar_button, invisible_toggle_sidebar_button, sync_database_button, import_database_button, export_database_button, task_tag_button, manage_task_tags_button, create_new_task_tags_button, cancel_create_new_task_tag_button, delete_task_tag_button, clear_task_needed_time_button, clear_task_due_date_button, add_due_date_button, edit_due_date_button, select_synchronization_filepath_button, clear_synchronization_filepath_button, date_formatting_button, color_palette_item_button, edit_color_palette_button, confirm_ok_button, confirm_cancel_button, search_tasks_button, cancel_search_tasks_button, settings_tab_button, import_source_code_todos_button, reimport_source_code_todos_button, show_source_code_todos_button, edit_project_name_button, start_timer_button, stop_timer_button, resume_timer_button, pause_timer_button, complete_task_timer_button, start_task_timer_button, import_google_tasks_button};

mod task_list;
pub use task_list::{task_list, TASK_LIST_ID, CREATE_NEW_TASK_NAME_INPUT_ID};

mod task_widget;
pub use task_widget::{task_widget, EDIT_NEEDED_TIME_TEXT_INPUT_ID, EDIT_DUE_DATE_TEXT_INPUT_ID};

mod task_tags_widget;
pub use task_tags_widget::task_tags_buttons;

mod duration_widget;
pub use duration_widget::{duration_widget, duration_text};

mod date_widget;
pub use date_widget::{date_text, days_left_widget};

mod loading_screen;
pub use loading_screen::loading_screen;

mod project_preview;
pub use project_preview::{project_preview, custom_project_preview, project_color_block};

mod seperator;
pub use seperator::{horizontal_seperator, horizontal_seperator_padded, vertical_seperator};

mod file_location;
pub use file_location::{filepath_widget, file_location};

mod modals;
pub use modals::{ConfirmModal, ConfirmModalMessage, ErrorMsgModal, ErrorMsgModalMessage};

mod settings_modal;
pub use settings_modal::{SettingsModal, SettingsModalMessage, SettingTab};

mod manage_task_tags_modal;
pub use manage_task_tags_modal::{ManageTaskTagsModal, ManageTaskTagsModalMessage};

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