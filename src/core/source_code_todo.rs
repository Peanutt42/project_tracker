use std::{collections::HashSet, fs::File, io::{self, BufRead}, path::{Path, PathBuf}};
use walkdir::{WalkDir, DirEntry};

use crate::core::Task;

pub fn import_source_code_todos(folders: Vec<PathBuf>) -> Vec<Task> {
	let mut todos = Vec::new();

	for folder_path in folders {
		if !should_import_source_code_todos_from_folder(&folder_path) {
			continue;
		}

		for entry in WalkDir::new(&folder_path).into_iter().filter_map(|e| e.ok()) {
			if should_import_source_code_todos_from_file(entry.path()) {
				import_source_code_todos_from_file(entry, &mut todos);
			}
		}
	}

	todos
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
			if folder_name.to_string_lossy().starts_with(".") {
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
	}
	else {
		true
	}
}

fn import_source_code_todos_from_file(entry: DirEntry, todos: &mut Vec<Task>) {
	if let Ok(file) = File::open(entry.path()) {
		for (i, line) in io::BufReader::new(file)
			.lines()
			.map_while(Result::ok)
			.enumerate()
		{
			let mut search_todo = |keyword: &'static str| {
				if let Some(index) =
					line.to_lowercase().find(&keyword.to_lowercase())
				{
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
						let line_number = i + 1;
						todos.push(Task::new(
							line.to_string(),
							format!("{source} on line {line_number}"),
							None,
							None,
							HashSet::new()
						));
					}
				}
			};

			// case insensitive!
			search_todo("// todo");
			search_todo("//todo");
			search_todo("# todo");
			search_todo("#todo");
		}
	}
}