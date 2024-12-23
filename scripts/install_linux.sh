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
sudo cp "$project_root/target/release/project_tracker" "/usr/bin/project_tracker"
sudo cp "$project_root/assets/icon.png" "/usr/share/icons/hicolor/512x512/apps/project_tracker.png"
sudo cp "$project_root/assets/project_tracker.desktop" "/usr/share/applications/project_tracker.desktop"

echo "Finished!"