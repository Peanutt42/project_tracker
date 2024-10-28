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

echo "Enabling systemd service..."
cd "$project_root/scripts"
database_file="/srv/project_tracker_server/database.json"
service_file="$project_root/target/release/ProjectTrackerServer.service"
echo "[Unit]" > "$service_file"
echo "Description=Runs the server to host the Project Tracker synchronization" >> "$service_file"
echo "" >> "$service_file"
echo "[Service]" >> "$service_file"
echo "ExecStart=$project_root/target/release/project_tracker_server $database_file" >> "$service_file"
echo "" >> "$service_file"
echo "[Install]" >> "$service_file"
echo "WantedBy=default.target" >> "$service_file"
sudo systemctl enable "$service_file"

echo "Starting service..."
sudo systemctl start ProjectTrackerServer.service

echo "Finished!"