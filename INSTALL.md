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
```

## Installation - Other
Project Tracker GUI/Client:
```bash
cargo r --release
```
Project Tracker Server:
```bash
cd project_tracker_server
cargo r --release
```