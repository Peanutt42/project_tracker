mod completion_bar;
pub use completion_bar::completion_bar;

mod buttons;
pub use buttons::{create_new_project_button, create_new_task_button, delete_project_button, delete_task_button, delete_all_done_tasks_button, show_done_tasks_button, cancel_create_project_button, cancel_create_task_button, overview_button, dangerous_button, theme_mode_button, settings_button, open_location_button, copy_to_clipboard_button, toggle_sidebar_button, invisible_toggle_sidebar_button, sync_database_button, task_tag_button, manage_task_tags_button, create_new_task_tags_button, cancel_create_new_task_tag_button, delete_task_tag_button};

mod task_list;
pub use task_list::{task_list, TASK_LIST_ID, CREATE_NEW_TASK_NAME_INPUT_ID};

mod task_widget;
pub use task_widget::{task_widget, EDIT_TASK_NAME_INPUT_ID};

mod task_tags_widget;
pub use task_tags_widget::task_tags_buttons;

mod loading_screen;
pub use loading_screen::loading_screen;

mod project_preview;
pub use project_preview::{project_preview, custom_project_preview, project_color_block};

mod seperator;
pub use seperator::{horizontal_seperator, colored_horizontal_seperator};

mod file_location;
pub use file_location::file_location;

mod modals;
pub use modals::{ConfirmModal, ConfirmModalMessage, ErrorMsgModal, ErrorMsgModalMessage};

mod switch_project_modal;
pub use switch_project_modal::{SwitchProjectModal, SwitchProjectModalMessage};

mod settings_modal;
pub use settings_modal::{SettingsModal, SettingsModalMessage};

mod manage_task_tags_modal;
pub use manage_task_tags_modal::{ManageTaskTagsModal, ManageTaskTagsModalMessage};

mod color_palette;
pub use color_palette::{color_palette, color_palette_item_button};

mod unfocusable;
pub use unfocusable::unfocusable;
