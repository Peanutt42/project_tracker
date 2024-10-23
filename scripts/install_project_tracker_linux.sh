#!/bin/bash

project_root="$(realpath "../")"

if [ "$project_root" = "" ]; then
	echo "failed to get absolute filepath of project root"
	exit 1
fi

cd "$project_root"

echo "Compiling..."

cargo b --release

cd "$project_root/scripts"

echo "Creating Desktop Entry..."

# Copy base .desktop entry
desktop_entry_filepath="$HOME/.local/share/applications/Project Tracker.desktop"

cp "Project Tracker Base.desktop" "$desktop_entry_filepath"

# Add user specific installation paths
echo "" >> "$desktop_entry_filepath"
echo "Exec=$project_root/target/release/project_tracker" >> "$desktop_entry_filepath"
echo "Icon=$project_root/assets/icon.svg" >> "$desktop_entry_filepath"

echo "Finished!"