# Fount Development Tasks

## Technical Debt and Refactoring

### Clean up the App state
The App struct is currently doing way too much and has become a monolithic container. I need to break it down into smaller, more manageable pieces. Specifically, I should pull out the editor logic (text, cursor, undo), the navigation data (scenes, characters), and the session stats (sprints, goals) into their own dedicated modules.

### Break down the Layout Engine
The build_layout function is a giant 400-line loop. It's becoming a nightmare to debug and extend. I need to split it up into specialized processors that handle specific things like page breaks, dialogue logic, scene numbering, and visual formatting markers.

### Speed up large document handling
Right now, the app re-processes the entire script whenever something changes. This is going to get slow once a script goes over 100 pages. I need to implement a tracking system so we only re-parse and re-layout the scenes or pages that were actually edited.

## Performance and Reliability

### Improve parser memory usage
The parser currently converts every line into a vector of characters for Unicode handling, which is wasteful for large files. I should switch to using something like unicode-segmentation for grapheme-aware iteration to cut down on redundant allocations.

### Better error handling
There are too many unwraps and unchecked options scattered throughout the code, especially in file I/O and command execution. I need to create a proper FountError type and migrate the logic to use Result patterns systematically so the app fails gracefully instead of crashing.

## Interface and Aesthetics

### Subtle formatting markers
Instead of hiding asterisks and underscores completely, I want to try rendering them in a very dim, low-contrast color. This would provide a hint about the document structure without being as distracting as the full-contrast markers.

### X-Ray pacing heatmap
It would be great to add a visual heatmap to the X-Ray view that shows the balance between dialogue and action across the script. This would help writers visualize the rhythm and pacing of their scenes at a glance.

## Features and Workflow

### Project support
I want to support a .fount file format that can group multiple Fountain files together. This would allow a writer to manage a script that is split into separate acts or episodes as a single unified workspace.

### Scripting and Plugins
Implement a Lua-based engine so users can write their own extensions for the editor. There is a separate architecture plan for this in the docs folder that covers the bridge API and event hooks.

## Finished Tasks
- Show match counts (X/Y) in the status bar during search.
- Added Alt+Up and Alt+Down for jumping between search results.
- Implemented a minimal tab bar for working with multiple open files.
- Updated the save command to prompt for a filename on new buffers.
- Added a visual indicator (asterisk) in the status bar for unsaved changes.
- Added sticky scene headings in the footer during scrolling.
- Added live scrolling preview while moving through the scene navigator.