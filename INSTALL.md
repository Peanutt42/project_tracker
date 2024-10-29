## Prerequisites - Build from source
- Rust (programing lanuage): https://www.rust-lang.org/tools/install
- On linux with the GUI/Client: install alsa development files, to play the notification sound:

	- Debian/Ubuntu:
		```bash
		sudo apt install libasound2-dev
		```
	- Fedora:
		```bash
		sudo dnf install alsa-lib-devel
		```


## Installation - Linux
Project Tracker GUI/Client:
```bash
cd scripts
./install_linux.sh
```
Project Tracker Server:
```bash
cd scripts
./install_server_linux.sh
./set_server_password_linux.sh
```

## Installation - Other
Project Tracker GUI/Client:
```bash
cargo r --release
```
Project Tracker Server:
"SERVER_DATA_DIRECTORY" specifies where the database and password files are stored
```bash
cd project_tracker_server
cargo r --release -- [SERVER_DATA_DIRECTORY]
```
> [!IMPORTANT]
> Make sure to set a different password for the server! The default password is: 1234
> To set the password, write the password in plain text into a "password.txt" file inside the "SERVER_DATA_DIRECTORY"