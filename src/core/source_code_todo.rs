use crate::Task;
use indexmap::IndexMap;
use project_tracker_core::TaskId;
use rayon::prelude::*;
use std::{
	collections::HashSet,
	fs::File,
	io::{self, BufRead},
	path::{Path, PathBuf},
};
use walkdir::{DirEntry, WalkDir};

pub fn import_source_code_todos(root_directory: PathBuf) -> IndexMap<TaskId, Task> {
	let todos: Vec<IndexMap<TaskId, Task>> = WalkDir::new(&root_directory)
		.into_iter()
		.par_bridge()
		.filter_map(|e| e.ok())
		.map(|entry| {
			if should_import_source_code_todos_from_file(entry.path()) {
				import_source_code_todos_from_file(entry)
			} else {
				IndexMap::new()
			}
		})
		.collect();

	let mut capacity = 0;
	for todos in todos.iter() {
		capacity += todos.len();
	}
	let mut source_code_todos = IndexMap::with_capacity(capacity);
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

	if let Some(parent_path) = filepath.parent() {
		should_import_source_code_todos_from_folder(parent_path)
	} else {
		true
	}
}

fn import_source_code_todos_from_file(entry: DirEntry) -> IndexMap<TaskId, Task> {
	if let Ok(file) = File::open(entry.path()) {
		let mut todos = IndexMap::new();

		for (line_index, line) in io::BufReader::new(file)
			.lines()
			.map_while(Result::ok)
			.enumerate()
		{
			let mut search_todo = |keyword: &'static str| {
				if let Some(index) = line.to_lowercase().find(&keyword.to_lowercase()) {
					let mut string_quotes_counter = 0;
					for c in line[0..index].chars() {
						if c == '\"' || c == '\'' {
							string_quotes_counter += 1;
						}
					}

					if string_quotes_counter % 2 == 0 {
						let line = line[index + keyword.len()..].to_string();
						let line = line.strip_prefix(':').unwrap_or(&line);
						let line = line.strip_prefix(' ').unwrap_or(line);
						let source = entry.path().display();
						let line_number = line_index + 1;
						let column_number = index + 1;
						todos.insert(
							TaskId::generate(),
							Task::new(
								line.to_string(),
								format!("{source}:{line_number}:{column_number}"),
								None,
								None,
								None,
								HashSet::new(),
							),
						);
					}
				}
			};

			// case insensitive!
			search_todo("// todo");
			search_todo("//todo");
			search_todo("# todo");
			search_todo("#todo");
		}

		todos
	} else {
		IndexMap::new()
	}
}
