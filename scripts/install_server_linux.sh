#!/bin/bash

project_root="$(realpath $(dirname $0)/../)"

if [ "$project_root" = "" ]; then
	echo "failed to get absolute filepath of project root"
	exit 1
fi

echo "Stopping service if already running"
sudo systemctl stop project_tracker_server.service >/dev/null 2>&1

echo "Compiling..."
cd "$project_root/project_tracker_server"
cargo b --release

echo "Installing files..."
binary_filepath="/usr/bin/project_tracker_server"
sudo cp "$project_root/target/release/project_tracker_server" "$binary_filepath"
server_data_directory="/srv/project_tracker_server"
sudo mkdir -p "$server_data_directory"

ORANGE='\033[0;33m'
RESET='\033[0;0m'

database_file="$server_data_directory/database.project_tracker"
if [ ! -f "$database_file" ]; then
	echo -e "${ORANGE}Missing database file!${RESET}"
	read -p "Create a empty database file? (yes/no): " choice
	case "$choice" in
		y|Y)
			cd "$project_root/project_tracker_core"
			cargo b --release --example create_empty_database
			sudo "$project_root/target/release/examples/create_empty_database" "$database_file"
			echo "Empty database file saved."
			;;
		n|N|*)
			;;
	esac
fi

if [ ! -f "$server_data_directory/password.txt" ]; then
	echo -e "${ORANGE}No password set!${RESET}"
	read -p "Set a new password? (yes/no): " choice
	case "$choice" in
		y|Y)
			cd "$project_root/scripts"
			./set_server_password_linux.sh
			;;
		n|N|*)
			;;
	esac
fi

echo "Enabling systemd service..."
sudo cp "$project_root/scripts/project_tracker_server.service" "/usr/lib/systemd/system/project_tracker_server.service"
sudo systemctl daemon-reload
sudo systemctl enable project_tracker_server.service >/dev/null 2>&1
sudo systemctl start project_tracker_server.service >/dev/null 2>&1

echo "Finished!"
