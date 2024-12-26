document.addEventListener("DOMContentLoaded", async () => {
	const home_button = document.getElementById("home_button");
	const logout_button = document.getElementById("logout_button");
	const admin_infos_div = document.getElementById("admin_infos_div");
	const cpu_usage_div = document.getElementById("cpu_usage_text");
	const cpu_temp_div = document.getElementById("cpu_temp_text");
	const ram_div = document.getElementById("ram_text");
	const uptime_div = document.getElementById("uptime_text");
	const connected_native_gui_clients_list = document.getElementById("connected_native_gui_clients_list");
	const connected_web_clients_list = document.getElementById("connected_web_clients_list");
	const latest_logs_text = document.getElementById("latest_logs_text");

	await populate_dom_with_admin_infos();

	home_button.addEventListener("click", open_home_page);

	logout_button.addEventListener("click", logout);

	// every 2s
	setInterval(populate_dom_with_admin_infos, 2000);

	function open_home_page() {
		window.location.href = "/";
	}

	function logout() {
		localStorage.removeItem("password");
		window.location.href = "/login";
	}

	async function populate_dom_with_admin_infos() {
		const admin_infos = await fetch_admin_infos();

		if (admin_infos) {
			cpu_usage_div.textContent = Math.round(admin_infos.cpu_usage * 100) + '%';
			if (admin_infos.cpu_temp) {
				cpu_temp_div.textContent = Math.round(admin_infos.cpu_temp) + ' °C';
			} else {
				cpu_temp_div.textContent = 'failed to get cpu temp!';
			}
			ram_div.textContent = admin_infos.ram_info;
			uptime_div.textContent = admin_infos.uptime;
			connected_native_gui_clients_list.innerHTML = "";
			for (address of admin_infos.connected_native_gui_clients) {
				const address_div = document.createElement("div");
				address_div.className = "address";
				address_div.textContent = address;
				connected_native_gui_clients_list.appendChild(address_div);
			}
			connected_web_clients_list.innerHTML = "";
			for (address of admin_infos.connected_web_clients) {
				const address_div = document.createElement("div");
				address_div.className = "address";
				address_div.textContent = address;
				connected_web_clients_list.appendChild(address_div);
			}

			const scrollToBottom = latest_logs_text.scrollTop === 0 ||
				latest_logs_text.scrollTop === latest_logs_text.scrollHeight;

			latest_logs_text.textContent = admin_infos.latest_logs_of_the_day;

			if (scrollToBottom) {
				latest_logs_text.scrollTop = latest_logs_text.scrollHeight;
			}
		} else {
			admin_infos_div.textContent = "Loading admin infos...";
		}
	}

	async function fetch_admin_infos() {
		const stored_password = localStorage.getItem("password");
		if (stored_password) {
			try {
				const password = stored_password;
				const response = await fetch("/admin_infos", {
					method: "POST",
					headers: { "Content-Type": "application/json" },
					body: JSON.stringify({ password }),
				});

				if (response.ok) {
					const admin_infos = await response.json();
					return admin_infos;
				}
				else {
					logout();
				}
			}
			catch (error) {
				logout();
			}
		}
		else {
			window.location.href = "/login";
		}
		return null;
	}
});