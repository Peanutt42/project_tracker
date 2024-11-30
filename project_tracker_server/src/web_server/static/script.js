const database_json_output = document.getElementById("database-json-output");
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

// auto login if last login was successful
document.addEventListener("DOMContentLoaded", () => {
	const stored_password = localStorage.getItem("password");
	if (stored_password) {
		hide_login_page();
		const last_loaded_database = localStorage.getItem("last_loaded_database");
		if (last_loaded_database) {
			database_json_output.textContent = last_loaded_database;
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
			const json = await response.json();
			const prettyJson = JSON.stringify(json, null, 2);
			database_json_output.textContent = prettyJson;
			hide_login_page();
			style_valid_password();
			localStorage.setItem("password", password);
			localStorage.setItem("last_loaded_database", prettyJson);
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
	database_json_output.textContent = "";
	show_login_page();
	localStorage.removeItem("password");
}

function connect_ws() {
	ws = new WebSocket('ws://' + location.host + '/modified');
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
	const stored_password = localStorage.getItem("password");
	if (stored_password) {
		login(stored_password);
	}
}