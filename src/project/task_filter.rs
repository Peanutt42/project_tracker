use crate::project::Task;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TaskFilter {
	#[default]
	All,
	Todo,
	Done,
}

impl TaskFilter {
	pub fn matches(self, task: &Task) -> bool {
		match self {
			TaskFilter::All => true,
			TaskFilter::Todo => !task.is_done(),
			TaskFilter::Done => task.is_done(),
		}
	}
}