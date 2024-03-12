# Giraffe

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg?style=flat-square)](https://www.gnu.org/licenses/gpl-3.0)
![actions](https://img.shields.io/github/actions/workflow/status/Linus-Mussmaecher/giraffe/continuous-testing.yml?label=tests&style=flat-square)
![commits](https://img.shields.io/github/commit-activity/m/Linus-Mussmaecher/giraffe?style=flat-square)
[![tech1](https://img.shields.io/badge/-Rust-000000?logo=rust&style=flat-square)](https://www.rust-lang.org/)
[![tech2](https://img.shields.io/badge/-Ratatui-000000?logo=gnome-terminal&style=flat-square)](https://ratatui.rs)


Giraffe is a terminal user interface (TUI) that displays markdown files right in your terminal.
Its main usecase is browsing and viewing a large folder structure of interlinked markdown files and - in conjunction with a text editor such as [Helix](https://helix-editor.com) or [Neovim](https://neovim.io) - serve as a replacement for note taking and knowledge base apps such as [Obsidian](https://obsidian.md) or [Notion](https://notion.so), but fully open-source and without leaving your terminal.

## Features

Giraffe is still heavily work-in-progress and not finished.
This list serves as a TODO-list of target features, not features implemented yet.
As soon as giraffe is in a usable state with core features implemented, this list will be split in available features and a TODO-list.

Giraffe features multiple screens to browse your markdown files.

### Selection Screen

 - Shows a list of all notes that is scrollable
 - List can be filtered by
  - Tags (all words of the filter string beginning with '#' are considered tags. Of these, either any or all, as by the user's choice, must be matched by the notes)
  - Title (must contain all words of the filter string not beginning with '#').
 - Shows the following statistics for each note in the list
  - Words
  - Characters
  - Total amount of outgoing links
  - Total amount of incoming links from all notes (_global inlinks_)
  - Total amount of incoming links from other notes matching the filter (_local inlinks_)
 - Also shows _global_ and _local_ statistics for all notes 
  - Total amount of notes
  - Total amount of unique tags
  - Total word count
  - Total character count
  - Total amount of outgoing links
  - Total amount of incoming links

### Viewing Screen
The main mode of giraffe.

 - Displays a single markdown note in your terminal. Supports all usual markdown features.
 - Also supports LaTeX-based equations. I am a mathematician and most of my files contain such formulas, that are sadly less readable in a raw format than normal markdown. This was one of the main reasons for creating giraffe.
 - Follow links within your files.
 - Styling of bold text etc. can be configured with a .toml file that contains serialized versions of the ratatui Style struct.

### Graph Screen

 - Shows a scrollable and zoomable graph of all your notes and their connections.
 - Allows showing note titles and viewing notes.

### Non-features

Giraffe is _not_ a markdown editor.
The program is read-only and allows viewing your files, but never changing them in any way.

There is no mouse support planned.

### Styling / Ricing

By default, giraffe takes on the colors defined by your terminal emulator.
These - and the way they are applied to the different markdown and UI elements - can be configured via .toml files in ```~/.config/giraffe```. 
See the upcoming wiki for more information.

## Platforms and Installation 

As of now, there are no official releases of giraffe, so the only way to download it and try it out is to clone this repository and compile it via ```cargo```.

Planned features include an upload to the Arch User Repository.

## Implementation

Giraffe is implemented in Rust, using the [ratatui](httsp://ratatui.rs) TUI library. 
LaTeX rendering is implemented via MathJax.
