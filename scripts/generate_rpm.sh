#!/bin/bash

project_root="$(realpath $(dirname $0)/../)"

if [ "$project_root" = "" ]; then
	echo "failed to get absolute filepath of project root"
	exit 1
fi

cd "$project_root"

echo "Installing 'cargo-generate-rpm'"
cargo install cargo-generate-rpm

echo "Compiling crate..."
cargo b --release

echo "Removing debug symbols..."
strip -s "$project_root/target/release/project_tracker"

echo "Generating rpm package..."
cargo generate-rpm

echo "Successfully generated rpm package inside 'target/generate-rpm/project_tracker-X.X.X-1.ARCH.rpm'"