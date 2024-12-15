const database_list = document.getElementById("database");
const logout_button = document.getElementById("logout_button");
const login_page = document.getElementById("login_page");
const password_input = document.getElementById("password_input");
const login_button = document.getElementById("login_button");
const show_password = document.getElementById("show_password");
const show_password_checkbox = document.getElementById("show_password_checkbox");
const invalid_password = document.getElementById("invalid_password");

let ws = null;
let reconnect_attempts = 0;
connect_ws();

logout_button.addEventListener("click", logout);

login_button.addEventListener("click", submit_password);

password_input.addEventListener("keypress", (event) => {
	if (event.key === "Enter") {
		submit_password();
	}
});

show_password.addEventListener("click", toggleShowPassword);
show_password_checkbox.addEventListener("click", toggleShowPassword);
show_password_checkbox.checked = false;

// auto logidatabase_listn if last login was successful
document.addEventListener("DOMContentLoaded", () => {
	const stored_password = localStorage.getItem("password");
	if (stored_password) {
		hide_login_page();
		const last_loaded_database = localStorage.getItem("last_loaded_database");
		if (last_loaded_database) {
			populate_dom_from_database(last_loaded_database);
		}
		login(stored_password);
	}
	else {
		show_login_page();
	}
});

function toggleShowPassword() {
	if (show_password_checkbox.checked) {
		password_input.type = "text";
	} else {
		password_input.type = "password";
	}
}

function show_login_page() {
	login_page.style.display = "block";
	logout_button.style.display = "none";
	password_input.value = "";
	show_password_checkbox.checked = false;
	style_valid_password();
}

function hide_login_page() {
	login_page.style.display = "none";
	logout_button.style.display = "block";
}

function style_invalid_password() {
	password_input.classList.add("invalid");
	invalid_password.style.display = "block";
}

function style_valid_password() {
	password_input.classList.remove("invalid");
	invalid_password.style.display = "none";
}

async function submit_password() {
	const password = password_input.value;
	await login(password);
}

async function login(password) {
	if (!password) {
		logout();
		alert("Please enter a password!");
		return;
	}

	try {
		const response = await fetch("/load_database", {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ password }),
		});

		if (response.ok) {
			const database = await response.json();
			populate_dom_from_database(database);
			hide_login_page();
			style_valid_password();
			localStorage.setItem("password", password);
			localStorage.setItem("last_loaded_database", database);
		} else if (response.status === 401) {
			logout();
			style_invalid_password();
		} else {
			logout();
			style_invalid_password();
			alert("An error occurred: " + response.statusText);
		}
	} catch (error) {
		logout();
		console.error("Failed to load database", error);
		alert("Failed to load database!");
	}
}

function logout() {
	database_list.innerHTML = "";
	show_login_page();
	localStorage.removeItem("password");
}

function populate_dom_from_database(database) {
	database_list.innerHTML = "";
	for (const project_id in database.projects) {
		const project = database.projects[project_id];
		const project_div = document.createElement("div");
		const project_name = document.createElement("div");
		project_name.className = "project_name";
		const done_task_count = Object.keys(project.done_tasks).length
		const all_task_count = done_task_count + Object.keys(project.todo_tasks).length + Object.keys(project.source_code_todos).length;
		project_name.textContent = project.name + " (" + done_task_count + '/' + all_task_count + ')';
		project_name.style.textDecorationColor = "rgb(" + project.color[0] + ", " + project.color[1] + ", " + project.color[2] + ")";
		project_div.appendChild(project_name);
		const task_list = document.createElement("ul");
		task_list.className = "task_list";

		const sorted_todo_tasks = field_ordered_tasklist_to_array(project.todo_tasks);
		sort_project_tasks(project.sort_mode, sorted_todo_tasks);
		for (const task_with_id of sorted_todo_tasks) {
			populate_dom_with_task(task_list, task_with_id[1], false);
		}
		const sorted_source_code_todos = field_ordered_tasklist_to_array(project.soure_code_todos);
		sort_project_tasks(project.sort_mode, sorted_source_code_todos);
		for (const task_with_id in sorted_source_code_todos) {
			populate_dom_with_task(task_list, task_with_id[1], false);
		}
		const done_task_list_section = document.createElement("details");
		done_task_list_section.className = "show_done_task_details";
		const show_done_tasks_summary = document.createElement("summary");
		show_done_tasks_summary.textContent = "Show done tasks";
		done_task_list_section.appendChild(show_done_tasks_summary);
		const done_task_list = document.createElement("ul");
		done_task_list.className = "task_list";
		sort_project_tasks(project.sort_mode, project.done_tasks);
		for (const task_with_id of project.done_tasks) {
			populate_dom_with_task(done_task_list, task_with_id[1], true);
		}
		done_task_list_section.appendChild(done_task_list);
		task_list.appendChild(done_task_list_section);
		project_div.appendChild(task_list);
		database_list.appendChild(project_div);
	}
}

function field_ordered_tasklist_to_array(tasks) {
	const array = [];
	for (const task_id in tasks) {
		array.push(
			[
				task_id,
				tasks[task_id],
			]
		);
	}
	return array;
}

function sort_project_tasks(sort_mode, tasks) {
	if (sort_mode === "DueDate") {
		tasks.sort((task_a_with_id, task_b_with_id) => {
			let task_a = task_a_with_id[1];
			let task_b = task_b_with_id[1];
			let due_date_a = task_a.due_date;
			let due_date_b = task_b.due_date;
			if (due_date_a) {
				if (due_date_b) {
					const date_a = new Date(due_date_a.year, due_date_a.month - 1, due_date_a.day); // months are 0-indexed
					const date_b = new Date(due_date_b.year, due_date_b.month - 1, due_date_b.day);
					return date_a - date_b;
				} else {
					return -1;
				}
			} else {
				if (due_date_b) {
					return 1;
				} else {
					return 0;
				}
			}
		});
	} else if (sort_mode === "NeededTime") {
		tasks.sort((task_a_with_id, task_b_with_id) => {
			let task_a = task_a_with_id[1];
			let task_b = task_b_with_id[1];
			let needed_time_a = task_a.needed_time_minutes;
			let needed_time_b = task_b.needed_time_minutes;
			if (needed_time_a) {
				if (needed_time_b) {
					return needed_time_a - needed_time_b;
				} else {
					return -1;
				}
			} else {
				if (needed_time_b) {
					return 1;
				} else {
					return 0;
				}
			}
		});
	}
}

function populate_dom_with_task(task_list, task, done) {
	const task_div = document.createElement("div");
	const checkbox = document.createElement("input");
	checkbox.type = "checkbox",
	checkbox.checked = done;
	checkbox.disabled = true; // TODO: check/uncheck tasks and send to server
	task_div.appendChild(checkbox);
	task_div.className = "task";

	let task_info = '';
	if (task.time_spend) {
		task_info += Math.floor(task.time_spend.offset_seconds / 60) + 'min';
		if (task.needed_time_minutes === null) {
			task_info += '/... - ';
		}
	}
	if (task.needed_time_minutes) {
		if (task.time_spend) {
			task_info += '/';
		}
		task_info += task.needed_time_minutes + 'min - ';
	}
	if (task.due_date) {
		task_info += task.due_date.day + '.' + task.due_date.month + '.' + task.due_date.year + ' - ';
	}
	task_info += task.name;
	const task_info_div = document.createElement("div");
	task_info_div.textContent = task_info;
	task_div.appendChild(task_info_div);
	task_list.appendChild(task_div);
}

function connect_ws() {
	ws = new WebSocket('wss://' + location.host + '/modified');
	ws.onopen = on_ws_open;
	ws.onclose = on_ws_close;
	ws.onmessage = on_ws_message;
}

function on_ws_open(event) {
	reconnect_attempts = 0;
}

function on_ws_close(event) {
	if (reconnect_attempts < 10) {
		reconnect_attempts++;
		console.log('reconnecting to modified ws endpoint... (' + reconnect_attempts + '. attempt)');
		// reconnect every 2 seconds
		setTimeout(connect_ws, 2000);
	}
	else {
		console.log('too many reconnect attempts, refresh when ws up again');
	}
}

// fetch updated database
function on_ws_message(msg) {
	populate_dom_from_database(JSON.parse(msg.data));
}