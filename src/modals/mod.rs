mod error_msg_modal;
pub use error_msg_modal::{ErrorMsgModal, ErrorMsgModalMessage};

mod confirm_modal;
pub use confirm_modal::{ConfirmModal, ConfirmModalMessage};

mod settings_modal;
pub use settings_modal::{SettingTab, SettingsModal, SettingsModalMessage};

mod manage_task_tags_modal;
pub use manage_task_tags_modal::{
	ManageTaskTagsModal, ManageTaskTagsModalAction, ManageTaskTagsModalMessage,
};

mod task_modal;
pub use task_modal::{TaskModal, TaskModalAction, TaskModalMessage};

mod create_task_modal;
pub use create_task_modal::{CreateTaskModal, CreateTaskModalAction, CreateTaskModalMessage};

mod wait_closing_modal;
pub use wait_closing_modal::{WaitClosingModal, WaitClosingModalMessage};
