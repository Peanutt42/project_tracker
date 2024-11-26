const database_json_output = document.getElementById("database-json-output");
const login_page = document.getElementById("login_page");
const password_input = document.getElementById("password_input");
const login_button = document.getElementById("login_button");
const show_password = document.getElementById("show_password");
const show_password_checkbox = document.getElementById("show_password_checkbox");
const invalid_password = document.getElementById("invalid_password");

login_button.addEventListener("click", submitPassword);

password_input.addEventListener("keypress", (event) => {
    if (event.key === "Enter") {
        submitPassword();
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

async function submitPassword() {
	const password = password_input.value;

    if (!password) {
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
		  login_page.style.display = "none";
		  password_input.classList.remove("invalid");
		  invalid_password.style.display = "none";
        } else if (response.status === 401) {
      		// TODO: show a error msg above password input and make input red etc.
			login_page.style.display = "block";
			password_input.classList.add("invalid");
			invalid_password.style.display = "block";
        } else {
			login_page.style.display = "block";
			password_input.classList.add("invalid");
			invalid_password.style.display = "block";
          alert("An error occurred: " + response.statusText);
        }
    } catch (error) {
        console.error("Failed to load database", error);
        alert("Failed to load database!");
    }
}