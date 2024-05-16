# 🌱 Rucola

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg?style=flat-square)](https://www.gnu.org/licenses/gpl-3.0)
![actions](https://img.shields.io/github/actions/workflow/status/Linus-Mussmaecher/rucola/continuous-testing.yml?label=tests&style=flat-square)
![commits](https://img.shields.io/github/commit-activity/m/Linus-Mussmaecher/rucola?style=flat-square)
[![tech1](https://img.shields.io/badge/-Rust-f74c00?logo=rust&style=flat-square)](https://www.rust-lang.org/)
[![tech2](https://img.shields.io/badge/-Ratatui-2563eb?logo=gnome-terminal&style=flat-square)](https://ratatui.rs)

Terminal-based browser and information aggregator for markdown file structures.

> [!CAUTION]
> This is a work-in-progress hobby project.
> Many Features are still lacking and bugs may appear frequently.
>
> All features described on this page are functional as far as I am aware, but the feature set is still incomplete.

## Contents
 - [Goals](#Goals)
 - [Installation](#installation)
 - [Features](#features)
    - [Overview Screen](#overview-screen)
       - [Statistics](#statistics)
       - [Filtering](#filtering)
       - [File Management](#file-management)
    - [Single-Note Screen](#single-note-screen)
    - [Configuration](#configuration)
 - [Planned Features](#planned-features)
 - [Technology & License](#technology-license)

## Goals
 - *Target audience*: Users of a [zettelkasten-style](https://en.wikipedia.org/wiki/Zettelkasten) note system of interlinked markdown notes.
 - Present the user with high-level information & statistics about their entire note set.
 - Show the same information about filtered subsets of notes, as well as their relation with the entire note set.
 - Allow the user to view link and backlink as well as statistical information about a single note.
 - Allow the user to make small edits (such as renaming or changing tags) from within the application, and open the note in more sophisticated, user-specified editors and viewers.
 - Provide all of this functionality without leaving the terminal.

## Installation
Currently, the only way to use this program is to clone this repository with
```
 git clone https://github.com/Linus-Mussmaecher/rucola
```
and install it via
```
 cargo install
```

For the future, a downloadable binary (for Windows) and releases to the AUR and Nix package registry are planned.

## Features

Rucola can be launched from the command line (`rucola`).
If no further arguments are given, rucola will open the notes in your default vault directory (specified in your config file).
This allows you to access your main vault of notes from anywhere in your file structure.
If no default vault is know yet, rucola will open in the current directory.
If you want to open a directory different from your default vault, you can pass the target as a positional argument, e.g. `rucola .` or `rucola ~/other/stuff`.

### Overview Screen

Rucola initially launches into the *overview screen*.
Here you will find an (unordered) list of all notes currently indexed by rucola, some statistics and a search bar.
The statistics refer to two environments:
 - The *global environment* consists of all notes currently indexed by rucola and can only be changed by restarting the program (or directly changing your files and reloading the screen).
 - The *local environment* consists of all notes currently matching your search query.

#### Statistics

The following statistics are shown for the environment:
 - The total number of notes contained.
 - The total number of words & characters in those notes.
 - The total number of (unique) tags used in those notes.
 - The number of broken links, i.e. links for which no target note could be found in the indexed structure.
 - For the global environment, the total number of links between notes.
 - For the local environemnt, links are again split up in three groups that can be used to judge how well-connected your local environment is in the set of all your notes:
    - *Internal links* have both source and target in the local envinroment.
    - *Incoming links* have their (valid) target in the local environment and a source in the global environment (may also be in the local environment).
    - *Outgoing links* have their source in the local environment and a (valid) target in the global environment (may also be in the local environment).
  
For the local statistics, all value are accompanied by a percentage value comparing it to the equivalent stat of the global environment.

Additionally, the following statistics are shown for every note in the filtered list:
 - The name.
 - The number of words & characters in the note.
 - The number of *global outlinks*, i.e. links that start in that node and have a valid target.
 - The number of *local outlinks*, i.e. global outlinks whose target is in the local environment.
 - The number of *global inlinks*, i.e. links from other notes whose target is that one.
 - The number of *local inlinks*, i.e. links from other notes within the local environment whose target is that one.
   
These statistics let you judge how well-connected a note is, and wether it is mostly relevant within the filtered context or in general.

#### Filtering
The filter line works in two stages: First, all tags found in your input (i.e. words starting with a `#` until the next whitespace) are matched with the notes' tags.
You can specify wether a note needs to have all entered tags to show up in the result, or if any of them is enough.
After that, the remaining filter string is fuzzy-matched with every note title, those without a match are excluded and the remaining notes are sorted according to their match score.

The list can also be sorted by any column, and the sorting can be switched between ascending and descending at will.

#### File Management
From the select view, you can access a couple of file management options for your notes:
 - Create a new note
 - Delete the selected note
 - Rename the selected note (with automatic link-renaming being a WIP)
 - Move the selected to another location relative to your current vault path
 - View/Edit the note in your configured text editor (such as a terminal based editor like vim or helix, or even obsidian).
   The used editor can be configured in the config file, if none is given, rucola defaults to your systems `$EDITOR` variable.


### Single-Note Screen
The single note screen shows a number of statistics about a single note:
 - Word count
 - Character count
 - Tags
 - Path

But more importantly, displays for that note lists of...
 - all *links*, i.e. notes linked to directly from the main note.
 - all *level 2 links*, i.e. notes linked to from notes that are in the *links* list.
 - all *backlinks*, i.e. notes that link directly to the main note.
 - all *level 2 backlinks*, i.e. notes linking to notes in the *backlinks* list.

This allows to you to get an overview about a note's connections in your network, and maybe find inspiration or unexpected correlations.

You can follow the links to given notes, and go back in your journey to previously visited notes.

### Configuration
Configuration files are - on Linux - stored in `XDG_CONFIGHOME/rucola`, which is usually `~/.config/rucola`.

## Planned Features
Planned features include:
 - More powerful search options, such as
   - Full text search through your files
   - Search for notes linked from or linking to a certain note
 - Integration of Obisidan URI for opening notes in obsidian
 - Performance improvements
 - Link updating on moving or renaming files
 - Auto-Fixing for broken links
 - Display of runtime errors within the program instead of frequent panics
 - Support nested tags in search

## Technology & License
Rucola is implemented using the [ratatui](https://ratatui.rs) framework in [Rust](https://www.rust-lang.org/) and released under the [GNU General Public License v3](https://www.gnu.org/licenses/gpl-3.0).
