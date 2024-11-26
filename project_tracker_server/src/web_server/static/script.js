document.getElementById("submit").addEventListener("click", submitPassword);

document.getElementById("password").addEventListener("keypress", (event) => {
    if (event.key === "Enter") {
        submitPassword();
    }
});

async function submitPassword() {
		const password = document.getElementById("password").value;

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
          document.getElementById("database-json-output").textContent = prettyJson;
        } else if (response.status === 401) {
      		// TODO: show a error msg above password input and make input red etc.
					document.getElementById("database-json-output").textContent = "Invalid password!";
        } else {
					document.getElementById("database-json-output").textContent = "An error occurred: " + response.statusText;
          alert("An error occurred: " + response.statusText);
        }
    } catch (error) {
        console.error("Failed to load database", error);
        alert("Failed to load database!");
    }
}