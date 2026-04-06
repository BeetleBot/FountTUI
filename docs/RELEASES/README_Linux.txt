# Fount for Linux

This is the portable binary of Fount, a terminal-based screenplay editor.

## Installation
To run Fount:
1. Extract the `fount` file from this archive.
2. Ensure it has execution permissions:
   $ chmod +x fount
3. Launch it:
   $ ./fount

## System Dependencies
Fount requires several standard Linux libraries locally. If the program fails to start, ensure the following are installed:
- libwayland-client
- libwayland-cursor
- libwayland-egl
- libdbus-1
- libgtk-3 (for file dialogue support)

### Portable Version (Musl)
If you're using the Musl binary (`fount-x86_64-unknown-linux-musl.tar.gz`), it is statically linked and should run on nearly any Linux distribution without additional libraries.
