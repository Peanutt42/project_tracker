## Install Client (deb and rpm, .exe packages available)
Just download the latest package from the [latest releases](https://github.com/Peanutt42/project_tracker/releases/latest)

<br>

## Install Server (deb and rpm packages available)
Just download the latest project_tracker_server-X.X.X-1.ARCH.rpm package from [latest releases](https://github.com/Peanutt42/project_tracker/releases/latest)

<br>

## Build from source - Prerequisites
- Rust (programing lanuage): https://www.rust-lang.org/tools/install
- OpenSSl development files (platform specific instructions below)
- C compiler (programming lanuage):
	- Linux: 'gcc'
	- Windows: 'msvc'
	- Macos: 'clang'

## Build from source - Client

### Linux
install openssl development files and alsa development files, to play the notification sound:
- Debian/Ubuntu:
	```bash
	sudo apt install libssl-dev libasound2-dev
	```
- Fedora:
	```bash
	sudo dnf install openssl-devel alsa-lib-devel
	```
to install for only the current user:
```bash
./scripts/install_linux.sh
```
to install system wide for all users:
```bash
./scripts/install_linux.sh --system-wide
```

### Windows
Run 'install_windows.bat' inside the 'scripts' folder as administrator

### Other platforms (no installation --> portable)
```bash
cargo r --release
```

### Generate deb package:
1. install needed packages to build the rust crate (see Linux/Debian development packages)
2. run generate script:
```bash
./scripts/generate_deb.sh
```
the deb package will be generated inside 'target/debian/project_tracker_<version>-1_<arch>.deb'

### Generate rpm package:
1. install needed packages to build the rust crate (see Linux/Fedora development packages)
2. run generate script:
```bash
./scripts/generate_rpm.sh
```
the rpm package will be generated inside 'target/generate-rpm/project_tracker-X.X.X-1.ARCH.rpm'

<br>

## Build from source - Server

### Prerequisites (excluding docker)
- OpenSSL (probably already installed): for https, self signed ssl certificate generation

<br>

> [!IMPORTANT]
> Make sure to set a different password for the server! The default password is: 1234
> for linux: just run '/scripts/set_password_linux.sh'
> other: write the password in plain text into a 'password.txt' file inside the 'SERVER_DATA_DIRECTORY'

### Local installation (linux only!)
with this method, you will be asked to set a new password automatically on first installation
```bash
cd scripts
./install_server_linux.sh
```

### Installation using docker
Build docker image:
```bash
sudo docker build -t project_tracker_server .
```
Run docker image:
Replace '/path/to/server/' with a actual directory to store persistent data like '/srv/project_tracker_server/'
```bash
sudo docker run -d -p 443:443 -p 8080:8080 -v /path/to/server/:/data project_tracker_server
```

### Installation - Other platforms
Project Tracker Server:
"SERVER_DATA_DIRECTORY" specifies where the database and password files are stored
```bash
cd project_tracker_server
cargo r --release -- [SERVER_DATA_DIRECTORY]
```

### Generate deb package (using cargo-deb):
1. install needed packages to build the rust crate (see Linux/Debian development packages)
2. run generate script
```bash
./scripts/generate_server_deb.sh
```
the deb package will be generated inside 'target/debian/project_tracker_server_<version>-1_<arch>.deb'

### Generate rpm package (using cargo-generate-rpm):
1. install needed packages to build the rust crate (see Linux/Fedora development packages)
2. run generate script
```bash
./scripts/generate_server.rpm.sh
```
the rpm package will be generated inside 'target/generate-rpm/project_tracker_server-X.X.X-1.ARCH.rpm'

<br>


### Configuration - Setting ssl certificates
By default, the server uses self signed generated ssl certificates for https.

If you are using something like [DuckDNS](https://duckdns.org) with [Nginx Proxy Manager](https://nginxproxymanager.com)
to have a verified ssl certificate for your selfhosted local server,
then download the ssl certificates generated and place the 'cert.pem' and 'key.pem' files
into the 'server_data_directory' directory (by default: '/srv/project_tracker_server/').

After restarting the server service (or just restarting the machine),
the provided certificates are used for the https web server.