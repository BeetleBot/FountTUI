#!/bin/sh

if ! [ $# = 0 ]; then
	echo "Fount .tar.gz installer"
	echo "Usage: sudo ./install.sh"
	exit
fi

echo "Installing Fount to /usr/share/fount..."

if ! [ $(id -u) = 0 ]; then
	echo "Please re-run using sudo: sudo ${0}" 
	exit 1
fi

echo "Copying program files to /usr/share/..."
cp -R usr/share/fount /usr/share/

echo "Creating symlink /usr/bin/fount..."
ln -sf /usr/share/fount/fount /usr/bin/fount

if [ -f "/usr/local/bin/fount" ] && ! [ -L "/usr/local/bin/fount" ]; then
	echo "Removing old binary from /usr/local/bin/fount to avoid PATH conflict..."
	rm -f /usr/local/bin/fount
fi

if command -v xdg-icon-resource >/dev/null 2>&1; then
	echo "Installing application icon..."
	icon="usr/share/icons/hicolor/256x256/apps/fount.png"
	if [ -f "$icon" ]; then
		xdg-icon-resource install --novendor --size 256 "$icon" fount
	fi
fi

if command -v xdg-desktop-menu >/dev/null 2>&1; then
	echo "Adding to menu..."
	xdg-desktop-menu install --novendor usr/share/applications/fount.desktop
elif [ -d "/usr/share/applications" ]; then
	echo "Installing .desktop file..."
	cp usr/share/applications/fount.desktop /usr/share/applications/
fi

echo "Fount installation complete!"
