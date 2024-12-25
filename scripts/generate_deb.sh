#!/bin/bash

project_root="$(realpath $(dirname $0)/../)"

if [ "$project_root" = "" ]; then
	echo "failed to get absolute filepath of project root"
	exit 1
fi

cd "$project_root"

echo "Installing 'cargo-deb'..."
cargo install cargo-deb

echo "Generate deb package..."
cargo deb

echo "Successfully generated deb package inside 'target/debian/project_tracker_<version>-1_<arch>.deb'"