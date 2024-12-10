#!/bin/bash

project_root="$(realpath $(dirname $0)/../)"

if [ "$project_root" = "" ]; then
	echo "failed to get absolute filepath of project root"
	exit 1
fi

echo "Stopping service if already running"
sudo systemctl stop ProjectTrackerServer.service >/dev/null 2>&1

echo "Compiling..."
cd "$project_root/project_tracker_server"
cargo b --release

echo "Installing files..."
binary_filepath="/usr/local/bin/project_tracker_server"
sudo cp "$project_root/target/release/project_tracker_server" "$binary_filepath"

echo "Enabling systemd service..."
cd "$project_root/scripts"
server_data_directory="/srv/project_tracker_server"
sudo mkdir -p "$server_data_directory"
service_file="$project_root/target/release/ProjectTrackerServer.service"
echo "[Unit]" > "$service_file"
echo "Description=Runs the server to host the Project Tracker synchronization" >> "$service_file"
echo "" >> "$service_file"
echo "[Service]" >> "$service_file"
echo "ExecStart=$binary_filepath $server_data_directory" >> "$service_file"
echo "" >> "$service_file"
echo "[Install]" >> "$service_file"
echo "WantedBy=default.target" >> "$service_file"
sudo systemctl enable "$service_file"

echo "Starting service..."
sudo systemctl start ProjectTrackerServer.service

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

echo "Finished!"
