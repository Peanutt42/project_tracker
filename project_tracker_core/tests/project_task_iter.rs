use std::collections::HashSet;

use project_tracker_core::{
	OrderedHashMap, Project, SerializableColor, SortMode, Task, TaskId, TaskType,
};

#[test]
fn task_iterator_visits_all_task_types() {
	let mut project = Project::new(
		"Test Project".to_string(),
		SerializableColor::default(),
		OrderedHashMap::new(),
		SortMode::Manual,
	);

	let todo_task_id = TaskId::generate();
	let source_code_task_id = TaskId::generate();
	let done_task_id = TaskId::generate();

	project.todo_tasks.insert(
		todo_task_id,
		Task::new(
			"Todo task".to_string(),
			String::new(),
			None,
			None,
			None,
			HashSet::new(),
		),
	);

	project.source_code_todos.insert(
		source_code_task_id,
		Task::new(
			"Source code todo".to_string(),
			"this_file_does_not_exit.txt".to_string(),
			None,
			None,
			None,
			HashSet::new(),
		),
	);

	project.done_tasks.insert(
		done_task_id,
		Task::new(
			"Done task".to_string(),
			String::new(),
			None,
			None,
			None,
			HashSet::new(),
		),
	);

	let mut iterated_task_count = 0;
	for (task_id, _task, task_type) in project.iter() {
		iterated_task_count += 1;
		match task_type {
			TaskType::Todo => assert_eq!(task_id, todo_task_id),
			TaskType::SourceCodeTodo => assert_eq!(task_id, source_code_task_id),
			TaskType::Done => assert_eq!(task_id, done_task_id),
		}
	}
	assert_eq!(iterated_task_count, project.total_tasks());
	assert_eq!(iterated_task_count, 3);
}
