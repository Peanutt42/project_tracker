#!/bin/bash

project_root="$(realpath "../")"

if [ "$project_root" = "" ]; then
	echo "failed to get absolute filepath of project root"
	exit 1
fi

cd "$project_root/project_tracker_server"

echo "Compiling..."

cargo b --release --features dashboard

cd "$project_root/scripts"

database_file="/srv/project_tracker_server/database.json"

echo "Enabling systemd service..."

service_file="$project_root/target/release/ProjectTrackerServer.service"

echo "[Unit]" > "$service_file"
echo "Description=Runs the server to host the Project Tracker synchronization and displays a dashboard gui" >> "$service_file"
echo "After=graphical.target" >> "$service_file"
echo "" >> "$service_file"
echo "[Service]" >> "$service_file"
echo "ExecStart=$project_root/target/release/project_tracker_server $database_file" >> "$service_file"
echo "User=$USER" >> "$service_file"
echo "Environment=DISPLAY=:0" >> "$service_file"
echo "Environment=XAUTHORITY=/home/$USER/.Xauthority" >> "$service_file"
echo "Environment=XDG_RUNTIME_DIR=/run/user/%U" >> "$service_file"
echo "" >> "$service_file"
echo "[Install]" >> "$service_file"
echo "WantedBy=graphical.target" >> "$service_file"

sudo systemctl enable "$service_file"

echo "Finished!"