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
installation_dir="$HOME/.local/project_tracker_server.app"
mkdir -p "$installation_dir"
binary_filepath="$installation_dir/bin/project_tracker_server"
mkdir -p "$installation_dir/bin"
cp "$project_root/target/release/project_tracker_server" "$binary_filepath"

echo "Enabling systemd service..."
cd "$project_root/scripts"
server_data_directory="/srv/project_tracker_server/"
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

echo "Finished!"