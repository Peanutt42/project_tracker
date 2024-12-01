#!/bin/bash

project_root="$(realpath $(dirname $0)/../)"

if [ "$project_root" = "" ]; then
	echo "failed to get absolute filepath of project root"
	exit 1
fi

cd "$project_root"

echo "Compiling..."
cargo b --release

echo "Installing files..."
installation_dir="$HOME/.local/project_tracker.app"
# Remove old installation
rm -rf "$installation_dir/"
mkdir -p "$installation_dir/"
# Copy binary
mkdir -p "$installation_dir/bin/"
binary_filepath="$installation_dir/bin/project_tracker"
cp "$project_root/target/release/project_tracker" "$binary_filepath"
# Copy icon
mkdir -p "$installation_dir/share/icons/hicolor/512x512/apps/"
icon_filepath="$installation_dir/share/icons/hicolor/512x512/apps/project_tracker.png"
cp "$project_root/assets/icon.png" "$icon_filepath"

echo "Creating Desktop Entry..."
# Copy base .desktop entry
cd "$project_root/scripts"
desktop_entry_filepath="$HOME/.local/share/applications/project_tracker.desktop"
mkdir -p "$HOME/.local/share/applications"
cp "Project Tracker Base.desktop" "$desktop_entry_filepath"
# Add user specific installation paths
echo "" >> "$desktop_entry_filepath"
echo "Exec=$binary_filepath" >> "$desktop_entry_filepath"
echo "Icon=$icon_filepath" >> "$desktop_entry_filepath"

echo "Finished!"