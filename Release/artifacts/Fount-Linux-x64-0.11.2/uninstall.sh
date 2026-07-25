#!/bin/sh

if ! [ $# = 0 ]; then
	echo "Fount .tar.gz uninstaller"
	echo "Usage: sudo ./uninstall.sh"
	exit
fi

echo "Uninstalling Fount..."

if ! [ $(id -u) = 0 ]; then
	echo "Please re-run using sudo: sudo ${0}" 
	exit 1
fi

echo "Removing program files..."
rm -rf /usr/share/fount

if [ -L "/usr/bin/fount" ]; then
	echo "Removing symlink..."
	rm /usr/bin/fount
fi

if [ -f "/usr/local/bin/fount" ]; then
	echo "Removing old binary from /usr/local/bin/fount..."
	rm -f /usr/local/bin/fount
fi

if command -v xdg-icon-resource >/dev/null 2>&1; then
	echo "Uninstalling application icon..."
	xdg-icon-resource uninstall --size 256 fount 2>/dev/null || true
fi

if command -v xdg-desktop-menu >/dev/null 2>&1; then
	echo "Removing from menu..."
	xdg-desktop-menu uninstall --novendor usr/share/applications/fount.desktop 2>/dev/null || true
fi

if [ -e "/usr/share/applications/fount.desktop" ]; then
	echo "Removing .desktop file..."
	rm -f /usr/share/applications/fount.desktop
fi

echo "Fount uninstallation complete!"
