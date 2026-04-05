#!/bin/bash
set -e

# Configuration
REPO="BeetleBot/Fount"
BINARY_NAME="fount"
INSTALL_DIR="/usr/local/bin"

# Detect OS
OS="$(uname -s)"
case "${OS}" in
    Linux*)     FILE_NAME="fount-portable-linux-x86_64.tar.gz";;
    Darwin*)    FILE_NAME="fount-macos-x86_64.tar.gz";;
    *)          echo "Unsupported OS: ${OS}"; exit 1;;
esac

# Get latest release tag
echo "Detecting latest version..."
LATEST_TAG=$(curl -s https://api.github.com/repos/${REPO}/releases/latest | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST_TAG" ]; then
    echo "Error: Could not find latest release."
    exit 1
fi

echo "Downloading ${BINARY_NAME} ${LATEST_TAG} for ${OS}..."
DOWNLOAD_URL="https://github.com/BeetleBot/Fount/releases/download/${LATEST_TAG}/${FILE_NAME}"

# Create temp directory
TEMP_DIR=$(mktemp -d)
trap "rm -rf ${TEMP_DIR}" EXIT

# Download and extract
curl -L "${DOWNLOAD_URL}" -o "${TEMP_DIR}/${FILE_NAME}"
tar -xzf "${TEMP_DIR}/${FILE_NAME}" -C "${TEMP_DIR}"

# Install
echo "Installing to ${INSTALL_DIR} (requires sudo)..."
sudo mv "${TEMP_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
sudo chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

echo "Successfully installed ${BINARY_NAME}!"
echo "Run 'fount' to start."
