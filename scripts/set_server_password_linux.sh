#!/bin/bash

password_filepath="/srv/project_tracker_server/password.txt"

echo "Stopping service if already running..."
sudo systemctl stop ProjectTrackerServer.service >/dev/null 2>&1
echo ""

echo "Please enter the password to protect the database."

read -sp "Password: " pswd

echo "writing the password to $password_filepath..."
printf "%s" "$pswd" | sudo tee "$password_filepath" > /dev/null
echo "Successfully set the new password!"

echo "Starting the service up again..."
sudo systemctl start ProjectTrackerServer.service >/dev/null 2>&1