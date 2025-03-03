#!/bin/bash

project_root="$(realpath $(dirname $0)/../)"

if [ "$project_root" = "" ]; then
	echo "failed to get absolute filepath of project root"
	exit 1
fi

cd "$project_root"

echo "Compiling..."
cargo b --release

if [ "$1" = "--system-wide" ]; then
	echo "Installing files system wide..."
	mkdir -p "/usr/bin"
	sudo cp "$project_root/target/release/project_tracker" "/usr/bin/project_tracker"
	mkdir -p "/usr/share/icons/hicolor/512x512/apps"
	sudo cp "$project_root/assets/icon.png" "/usr/share/icons/hicolor/512x512/apps/project_tracker.png"
	mkdir -p "/usr/share/applications"
	sudo cp "$project_root/scripts/project_tracker_system_wide.desktop" "/usr/share/applications/project_tracker.desktop"
else
	echo "Installing files locally for only the current user..."
	mkdir -p "$HOME/.local/bin"
	cp "$project_root/target/release/project_tracker" "$HOME/.local/bin/project_tracker"
	mkdir -p "$HOME/.local/project_tracker.app/icons/hicolor/512x512/apps"
	cp "$project_root/assets/icon.png" "$HOME/.local/project_tracker.app/icons/hicolor/512x512/apps/project_tracker.png"
	mkdir -p "$HOME/.local/share/applications"
	cp "$project_root/scripts/project_tracker_local_user.desktop" "$HOME/.local/share/applications/project_tracker.desktop"
	sed -i "s|\$HOME|$HOME|g" "$HOME/.local/share/applications/project_tracker.desktop" # replaces '$HOME' with '/home/user'
fi

echo "Finished!"