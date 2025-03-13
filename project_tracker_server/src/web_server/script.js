document.addEventListener("DOMContentLoaded", () => {
	// register pwa service-worker
	if ("serviceWorker" in navigator) {
		navigator.serviceWorker.register("/service-worker.js").then(() => {
			console.log("Service Worker Registered");
		});
	}

	const project_list = document.getElementById("project_list");
	const task_list = document.getElementById("task_list");
	let touch_start_x = 0;
	let swiping = false;
	task_list.addEventListener("touchstart", (e) => {
		touch_start_x = e.touches[0].clientX;
		swiping = true;
	});
	task_list.addEventListener("touchmove", (e) => {
		if (!swiping) {
			return;
		}

		const touch_end_x = e.changedTouches[0].clientX;
		const delta_x = touch_end_x - touch_start_x;
		const window_width = window.outerWidth;

		if (Math.abs(delta_x) / window_width >= 0.25) {
			const last_loaded_database = JSON.parse(
				localStorage.getItem("last_loaded_database"),
			);
			if (last_loaded_database) {
				swiping = false;
				swipe_projects(last_loaded_database, delta_x > 0);
			}
		}
	});
	task_list.addEventListener("touchend", () => {
		swiping = false;
	});

	const database_list = document.getElementById("database");
	const admin_dashboard_button = document.getElementById(
		"admin_dashboard_button",
	);
	const logout_button = document.getElementById("logout_button");
	const offline_indicator = document.getElementById("offline_indicator");
	offline_indicator.style.display = "none";

	const create_task_name_input = document.getElementById(
		"create_task_name_input",
	);
	create_task_name_input.addEventListener("keypress", (event) => {
		if (event.key == "Enter") {
			create_task();
		}
	});
	const create_task_name_button = document.getElementById(
		"create_task_name_button",
	);
	create_task_name_button.addEventListener("click", () => {
		create_task();
	});

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
			const response = await fetch("/api/load_database", {
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

	function create_task() {
		if (ws) {
			let selected_project_id = localStorage.getItem("selected_project_id");
			if (selected_project_id) {
				ws.send(
					JSON.stringify({
						CreateTask: {
							project_id: selected_project_id,
							task_name: create_task_name_input.value,
						},
					}),
				);
				create_task_name_input.value = "";
			}
		}
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

	// switch to the left/right project of the currently selected project
	function swipe_projects(database, left) {
		const selected_project_id = localStorage.getItem("selected_project_id");
		let new_selected_project_index = 0;
		if (selected_project_id) {
			const selected_project_index =
				Object.keys(database).indexOf(selected_project_id);
			if (selected_project_index !== -1) {
				if (left && selected_project_index > 0) {
					new_selected_project_index = selected_project_index - 1;
				} else if (
					!left &&
					selected_project_index < Object.keys(database).length - 1
				) {
					new_selected_project_index = selected_project_index + 1;
				} else {
					new_selected_project_index = selected_project_index;
				}
			}
		}
		const new_selected_project_id =
			Object.keys(database)[new_selected_project_index];
		localStorage.setItem("selected_project_id", new_selected_project_id);
		populate_dom_from_database(database);
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

		const task_content_div = document.createElement("div");
		task_content_div.className = "task_content";
		task_content_div.appendChild(task_tags_and_name_div);

		if (task.description && task.description.trim().length > 0) {
			const task_description_section = document.createElement("details");
			task_description_section.style.width = "fit-content";

			const task_description_summary = document.createElement("summary");
			const task_description_icon = document.createElement("img");
			task_description_icon.src = "/static/justify-left.svg";
			task_description_summary.append(task_description_icon);
			const task_description_div = document.createElement("div");
			task_description_div.className = "task_description";
			task_description_div.style.display = "none";
			task_description_div.textContent = task.description;
			task_description_summary.appendChild(task_description_div);
			task_description_section.append(task_description_summary);

			task_description_section.addEventListener("toggle", () => {
				if (task_description_section.open) {
					task_description_icon.style.display = "none";
					task_description_div.style.display = "block";
				} else {
					task_description_icon.style.display = "block";
					task_description_div.style.display = "none";
				}
			});

			task_content_div.appendChild(task_description_section);
		}

		task_div.appendChild(task_content_div);

		task_list.appendChild(task_div);
	}

	function connect_ws() {
		console.log("connecting to ws...");
		ws = new WebSocket("wss://" + location.host + "/api/ws/");
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
			const database = JSON.parse(msg.data);
			populate_dom_from_database(database);
			localStorage.setItem("last_loaded_database", JSON.stringify(database));
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
