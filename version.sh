#!/bin/bash
set -e

# Set variables
VERSION=$1
RELEASE_DATE=$(date +"%Y-%m-%d")

# Update Cargo version
sed -i "s/^version = .*$/version = \"$VERSION\"/" Cargo.toml
