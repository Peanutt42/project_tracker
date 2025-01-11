document.addEventListener("DOMContentLoaded", () => {
	const password_input = document.getElementById("password_input");
	const login_button = document.getElementById("login_button");
	const show_password = document.getElementById("show_password");
	const show_password_checkbox = document.getElementById(
		"show_password_checkbox",
	);
	const invalid_password = document.getElementById("invalid_password");

	login_button.addEventListener("click", submit_password);

	password_input.addEventListener("keypress", (event) => {
		if (event.key === "Enter") {
			submit_password();
		}
	});

	show_password.addEventListener("click", toggleShowPassword);
	show_password_checkbox.addEventListener("click", toggleShowPassword);
	show_password_checkbox.checked = false;

	function toggleShowPassword() {
		if (show_password_checkbox.checked) {
			password_input.type = "text";
		} else {
			password_input.type = "password";
		}
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
				style_valid_password();
				localStorage.setItem("password", password);
				localStorage.setItem("last_loaded_database", JSON.stringify(database));
				window.location.href = "/";
			} else if (response.status === 401) {
				style_invalid_password();
				console.error("invalid password, unauthorized!");
			} else {
				style_invalid_password();
				console.error("invalid response!");
			}
		} catch (error) {
			console.error("failed to fetch response: " + error + "!");
			style_invalid_password();
		}
	}

	function logout() {
		localStorage.removeItem("password");
	}
});
