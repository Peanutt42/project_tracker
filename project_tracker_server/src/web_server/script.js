document.addEventListener("DOMContentLoaded", () => {
	// register pwa service-worker
	if ("serviceWorker" in navigator) {
		navigator.serviceWorker.register("/service-worker.js").then(() => {
			console.log("Service Worker Registered");
		});
	}

	const project_list = document.getElementById("project_list");
	const task_list = document.getElementById("task_list");

	const database_list = document.getElementById("database");
	const admin_dashboard_button = document.getElementById(
		"admin_dashboard_button",
	);
	const logout_button = document.getElementById("logout_button");
	const offline_indicator = document.getElementById("offline_indicator");
	offline_indicator.style.display = "none";

	window.addEventListener("offline", () => {
		offline_indicator.style.display = "block";
	});

	let ws = null;
	let ws_authenticated = false;
	let reconnect_attempts = 0;

	admin_dashboard_button.addEventListener("click", open_admin_page);

	logout_button.addEventListener("click", logout);

	const stored_password = localStorage.getItem("password");
	if (stored_password) {
		const last_loaded_database = JSON.parse(
			localStorage.getItem("last_loaded_database"),
		);
		if (last_loaded_database) {
			populate_dom_from_database(last_loaded_database);
		}

		login(stored_password);

		connect_ws();
	} else {
		window.location.href = "/login";
	}

	async function login(password) {
		if (!password) {
			logout();
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
				localStorage.setItem("password", password);
				localStorage.setItem("last_loaded_database", JSON.stringify(database));
				populate_dom_from_database(database);
			} else if (response.status === 401) {
				console.error("invalid password, unauthorized!");
				logout();
			} else {
				console.error("invalid response!");
			}
		} catch (error) {
			console.error("failed to fetch response: " + error + "!");
		}
	}

	function logout() {
		localStorage.removeItem("password");
		window.location.href = "/login";
	}

	function open_admin_page() {
		window.location.href = "/admin";
	}

	function color_to_str(color) {
		return "rgb(" + color[0] + ", " + color[1] + ", " + color[2] + ")";
	}

	function populate_dom_from_database(database) {
		let selected_project_id = localStorage.getItem("selected_project_id");
		if (selected_project_id === null) {
			selected_project_id = Object.keys(database)[0];
		}
		localStorage.setItem("selected_project_id", selected_project_id);

		project_list.innerHTML = "";
		for (const project_id in database) {
			const selected = selected_project_id === project_id;
			const project = database[project_id];

			const project_button = document.createElement("button");
			const project_color_str = color_to_str(project.color);
			project_button.style.color = project_color_str;
			if (selected) {
				project_button.style.background = project_color_str;
			} else {
				project_button.style.background = "none";
				project_button.style.borderStyle = "solid";
				project_button.style.borderColor = project_color_str;
				project_button.style.borderWidth = "1px";
			}
			project_button.addEventListener("click", () => {
				localStorage.setItem("selected_project_id", project_id);
				populate_dom_from_database(database);
			});
			const project_name = document.createElement("div");
			const project_color_brightness =
				0.2126 * project.color[0] +
				0.7152 * project.color[1] +
				0.0722 * project.color[2];
			if (selected && project_color_brightness > 255 * 0.6) {
				project_name.style.color = "black";
			} else {
				project_name.style.color = "white";
			}
			project_name.textContent = project.name;
			project_button.appendChild(project_name);
			project_list.appendChild(project_button);
		}

		populate_dom_from_project(
			selected_project_id,
			database[selected_project_id],
		);
	}

	function populate_dom_from_project(project_id, project) {
		task_list.innerHTML = "";

		const project_div = document.createElement("div");
		project_div.className = "project";
		const project_name = document.createElement("div");
		project_name.className = "project_name";
		const done_task_count = Object.keys(project.done_tasks).length;
		const all_task_count =
			done_task_count +
			Object.keys(project.todo_tasks).length +
			Object.keys(project.source_code_todos).length;
		project_name.textContent =
			project.name + " (" + done_task_count + "/" + all_task_count + ")";
		project_name.style.textDecorationColor = color_to_str(project.color);
		project_div.appendChild(project_name);

		const sorted_todo_tasks = field_ordered_tasklist_to_array(
			project.todo_tasks,
		);
		sort_project_tasks(project.sort_mode, sorted_todo_tasks);
		for (const task_with_id of sorted_todo_tasks) {
			populate_dom_with_task(
				project_id,
				task_with_id[0],
				task_list,
				task_with_id[1],
				project.task_tags,
				false,
			);
		}
		const sorted_source_code_todos = field_ordered_tasklist_to_array(
			project.soure_code_todos,
		);
		sort_project_tasks(project.sort_mode, sorted_source_code_todos);
		for (const task_with_id in sorted_source_code_todos) {
			populate_dom_with_task(
				project_id,
				task_with_id[0],
				task_list,
				task_with_id[1],
				project.task_tags,
				false,
			);
		}
		const done_task_list_section = document.createElement("details");
		done_task_list_section.className = "show_done_task_details";
		const show_done_tasks_summary = document.createElement("summary");
		show_done_tasks_summary.textContent = "Show done tasks";
		done_task_list_section.appendChild(show_done_tasks_summary);
		const done_task_list = document.createElement("ul");
		done_task_list.className = "task_list";
		const sorted_done_tasks = field_ordered_tasklist_to_array(
			project.done_tasks,
		);
		sort_project_tasks(project.sort_mode, sorted_done_tasks);
		for (const task_with_id of sorted_done_tasks) {
			populate_dom_with_task(
				project_id,
				task_with_id[0],
				done_task_list,
				task_with_id[1],
				project.task_tags,
				true,
			);
		}
		done_task_list_section.appendChild(done_task_list);
		task_list.appendChild(done_task_list_section);
	}

	function task_tag_dom(task_tag) {
		const tag_div = document.createElement("div");
		tag_div.className = "tag";
		tag_div.textContent = task_tag.name;
		tag_div.style.borderColor = color_to_str(task_tag.color);
		return tag_div;
	}

	function field_ordered_tasklist_to_array(tasks) {
		const array = [];
		for (const task_id in tasks) {
			array.push([task_id, tasks[task_id]]);
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
						const date_a = new Date(
							due_date_a.year,
							due_date_a.month - 1,
							due_date_a.day,
						); // months are 0-indexed
						const date_b = new Date(
							due_date_b.year,
							due_date_b.month - 1,
							due_date_b.day,
						);
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

	function populate_dom_with_task(
		project_id,
		task_id,
		task_list,
		task,
		task_tags,
		done,
	) {
		const task_div = document.createElement("div");
		task_div.className = "task";

		const checkbox = document.createElement("input");
		(checkbox.type = "checkbox"), (checkbox.checked = done);
		checkbox.addEventListener("click", () => {
			if (ws) {
				ws.send(
					JSON.stringify({
						ToggleTask: {
							project_id: project_id,
							task_id: task_id,
							checked: checkbox.checked,
						},
					}),
				);
			}
		});
		task_div.appendChild(checkbox);

		const task_tags_and_name_div = document.createElement("div");
		task_tags_and_name_div.style.display = "flex";
		task_tags_and_name_div.style.flexDirection = "column";

		const task_tags_list = document.createElement("ul");
		task_tags_list.className = "tag_list";
		for (const [tag_id, tag] of Object.entries(task_tags)) {
			if (task.tags.includes(tag_id)) {
				task_tags_list.appendChild(task_tag_dom(tag));
			}
		}
		task_tags_and_name_div.appendChild(task_tags_list);

		let task_info = "";
		if (task.time_spend) {
			task_info += Math.floor(task.time_spend.offset_seconds / 60) + "min";
			if (task.needed_time_minutes === null) {
				task_info += "/... - ";
			}
		}
		if (task.needed_time_minutes) {
			if (task.time_spend) {
				task_info += "/";
			}
			task_info += task.needed_time_minutes + "min - ";
		}
		if (task.due_date) {
			task_info +=
				task.due_date.day +
				"." +
				task.due_date.month +
				"." +
				task.due_date.year +
				" - ";
		}
		task_info += task.name;

		const task_info_div = document.createElement("div");
		task_info_div.textContent = task_info;

		task_tags_and_name_div.appendChild(task_info_div);

		task_div.appendChild(task_tags_and_name_div);

		task_list.appendChild(task_div);
	}

	function connect_ws() {
		console.log("connecting to ws...");
		ws = new WebSocket("wss://" + location.host + "/ws/");
		ws.onopen = on_ws_open;
		ws.onclose = on_ws_close;
		ws.onmessage = on_ws_message;
		ws_authenticated = false;
	}

	function on_ws_open(event) {
		reconnect_attempts = 0;
		const password = localStorage.getItem("password");
		if (password) {
			ws.send(
				JSON.stringify({
					password: password,
				}),
			);
		}
	}

	function on_ws_close(event) {
		setTimeout(() => {
			offline_indicator.style.display = "block";
		}, 500);

		if (reconnect_attempts < 10) {
			reconnect_attempts++;
			console.log(
				"reconnecting to modified ws endpoint... (" +
					reconnect_attempts +
					". attempt)",
			);
			// reconnect every 2 seconds
			setTimeout(connect_ws, 2000);
		} else {
			console.log("too many reconnect attempts, refresh when ws up again");
		}
	}

	// fetch updated database
	function on_ws_message(msg) {
		if (ws_authenticated) {
			populate_dom_from_database(JSON.parse(msg.data));
		} else {
			const authentication_response = JSON.parse(msg.data);
			if (authentication_response.successfull) {
				ws_authenticated = true;
				offline_indicator.style.display = "none";
			} else {
				logout();
			}
		}
	}
});
