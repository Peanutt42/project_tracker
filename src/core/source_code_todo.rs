use crate::Task;
use project_tracker_core::{OrderedHashMap, TaskId};
use rayon::prelude::*;
use std::{
	collections::BTreeSet,
	fs::File,
	io::{self, BufRead},
	path::{Path, PathBuf},
};
use tracing::debug;
use walkdir::{DirEntry, WalkDir};

pub fn import_source_code_todos(root_directory: PathBuf) -> OrderedHashMap<TaskId, Task> {
	let todos: Vec<OrderedHashMap<TaskId, Task>> = WalkDir::new(&root_directory)
		.into_iter()
		.par_bridge()
		.filter_map(|e| e.ok())
		.map(|entry| {
			if should_import_source_code_todos_from_file(entry.path()) {
				import_source_code_todos_from_file(entry)
			} else {
				OrderedHashMap::new()
			}
		})
		.collect();

	let mut capacity = 0;
	for todos in todos.iter() {
		capacity += todos.len();
	}
	let mut source_code_todos = OrderedHashMap::with_capacity(capacity);
	for mut todos in todos {
		source_code_todos.append(&mut todos);
	}
	source_code_todos
}

fn should_import_source_code_todos_from_folder(folder_path: &Path) -> bool {
	if !folder_path.is_dir() {
		return false;
	}

	for ancestor in folder_path.ancestors() {
		if !ancestor.is_dir() {
			continue;
		}

		if let Some(folder_name) = ancestor.file_name() {
			let folder_name_str = folder_name.to_string_lossy();
			if folder_name_str.starts_with(".") || folder_name_str == "target" {
				return false;
			}
		}
	}

	true
}

fn should_import_source_code_todos_from_file(filepath: &Path) -> bool {
	if filepath.is_dir() {
		return false;
	}

	match filepath.parent() {
		Some(parent_path) => should_import_source_code_todos_from_folder(parent_path),
		None => true,
	}
}

// TODO: support multiline comments (--> 2.. lines into description?)
fn import_source_code_todos_from_file(entry: DirEntry) -> OrderedHashMap<TaskId, Task> {
	let filepath = entry.path();

	match File::open(filepath) {
		Ok(file) => {
			let mut todos = OrderedHashMap::new();

			for (line_index, line) in io::BufReader::new(file)
				.lines()
				.map_while(Result::ok)
				.enumerate()
			{
				if let Some((name, column_number)) = import_source_code_comment_from_line(line) {
					let source = entry.path().display();
					let line_number = line_index + 1;
					todos.insert(
						TaskId::generate(),
						Task::new(
							name,
							format!("{source}:{line_number}:{column_number}"),
							None,
							None,
							None,
							BTreeSet::new(),
						),
					);
				}
			}

			todos
		}
		Err(_) => {
			debug!(
				"could not open source code file in '{}'",
				filepath.display()
			);
			OrderedHashMap::new()
		}
	}
}

// (todo_name, column_number)
fn import_source_code_comment_from_line(line: String) -> Option<(String, usize)> {
	let line_lowercase = line.to_lowercase();

	let search_todo_comment = |keyword: &'static str| {
		if let Some(index) = line_lowercase.find(&keyword.to_lowercase()) {
			let mut string_quotes_counter = 0;
			for c in line[0..index].chars() {
				if c == '\"' || c == '\'' {
					string_quotes_counter += 1;
				}
			}

			if string_quotes_counter % 2 == 0 {
				let todo_name = line[index + keyword.len() + 1..].to_string();
				let todo_name = todo_name.strip_prefix(':').unwrap_or(&todo_name);
				let todo_name = todo_name.strip_prefix(' ').unwrap_or(todo_name);
				let column_number = index + 1;
				return Some((todo_name.to_string(), column_number));
			}
		}
		None
	};

	let search_todo_macro = {
		const TODO_MACRO_KEYWORD_START: &str = "todo!(";
		if let Some(index) = line_lowercase.find(TODO_MACRO_KEYWORD_START) {
			let mut string_quotes_counter = 0;
			for c in line[0..index].chars() {
				if c == '\"' || c == '\'' {
					string_quotes_counter += 1;
				}
			}
			if string_quotes_counter % 2 == 0 {
				let todo_macro_arg_start_index = index + TODO_MACRO_KEYWORD_START.len();
				let todo_name = match line.chars().nth(todo_macro_arg_start_index) {
					Some('\"') => {
						let todo_name_start_index = todo_macro_arg_start_index + 1;
						let mut todo_name_end_index = todo_name_start_index;
						if let Some(i) = line[todo_name_start_index..].find('\"') {
							todo_name_end_index = todo_name_start_index + i;
						}
						line[todo_name_start_index..todo_name_end_index].to_string()
					}
					_ => String::new(),
				};
				let column_number = index + 1;
				return Some((todo_name.to_string(), column_number));
			}
		}
		None
	};

	// case insensitive!
	search_todo_comment("// todo")
		.or(search_todo_comment("//todo"))
		.or(search_todo_comment("# todo"))
		.or(search_todo_comment("#todo"))
		.or(search_todo_macro)
}
