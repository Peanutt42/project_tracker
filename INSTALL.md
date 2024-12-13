## Prerequisites
- Rust (programing lanuage): https://www.rust-lang.org/tools/install


## Client

### Linux
install alsa development files, to play the notification sound:
- Debian/Ubuntu:
	```bash
	sudo apt install libasound2-dev
	```
- Fedora:
	```bash
	sudo dnf install alsa-lib-devel
	```
to install locally:
```bash
cd scripts
./install_linux.sh
```

### Windows
```bat
cd .\scripts\
.\install_windows.bat
```

### Other platforms (no installation --> portable)
```bash
cargo r --release
```


## Server

> [!IMPORTANT]
> Make sure to set a different password for the server! The default password is: 1234
> for linux: just run '/scripts/set_password_linux.sh'
> other: write the password in plain text into a 'password.txt' file inside the 'SERVER_DATA_DIRECTORY'

### Local installation (linux only!)
```bash
cd scripts
./install_server_linux.sh
```

### Installation using docker
Build docker image:
```bash
sudo docker build -t project_tracker .
```
Run docker image:
Replace '/path/to/server/' with a actual directory to store persistent data like '/srv/project_tracker_server'
```bash
sudo docker run -d -p 80:80 -p 8080:8080 -v /path/to/server/:/data project_tracker
```

### Installation - Other platforms
Project Tracker Server:
"SERVER_DATA_DIRECTORY" specifies where the database and password files are stored
```bash
cd project_tracker_server
cargo r --release -- [SERVER_DATA_DIRECTORY]
```
