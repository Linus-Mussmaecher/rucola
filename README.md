# 🌱 Rucola

[<img alt="github" src="https://img.shields.io/badge/github-Linus--Mussmaecher/rucola-8da0cb?style=for-the-badge&labelColor=555555&color=8da0cb&logo=github">](https://github.com/Linus-Mussmaecher/rucola)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg?labelColor=555555&style=for-the-badge&logo=gnu)](https://www.gnu.org/licenses/gpl-3.0)
[<img alt="test status" src="https://img.shields.io/github/actions/workflow/status/Linus-Mussmaecher/rucola/continuous-testing.yml?label=tests&branch=main&labelColor=555555&style=for-the-badge">](https://github.com/Linus-Mussmaecher/rucola/actions?query=branch%3Amain)
[<img alt="commit status" src="https://img.shields.io/github/commit-activity/m/Linus-Mussmaecher/rucola?style=for-the-badge&labelColor=555555&color=66c2a5">](https://github.com/Linus-Mussmaecher/rucola/commits/main)
[<img alt="rust" src="https://img.shields.io/badge/Rust-2021_Edition-ed9974?labelColor=555555&logo=rust&style=for-the-badge">](https://www.rust-lang.org/)
[<img alt="ratatui" src="https://img.shields.io/badge/Ratatui-0.26-7fa4f5?labelColor=555555&logo=gnome-terminal&style=for-the-badge">](https://www.ratatui.rs/)

Terminal-based browser and information aggregator for markdown file structures.

> [!CAUTION]
> This is a work-in-progress hobby project.
> All features described on this page are functional, but many are still lacking and bugs may appear frequently.

## Contents
 - [Goals](#Goals)
 - [Installation](#installation)
 - [Features](#features)
    - [Select Screen](#select-screen)
       - [Statistics](#statistics)
       - [Filtering](#filtering)
       - [File Management](#file-management)
    - [Display Screen](#display-screen)
    - [Configuration](#configuration)
 - [Planned Features](#planned-features)
 - [Technology](#technology)
 - [License](#license)

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
 cargo install --path .
```

For the future, a downloadable binary (for Windows) and releases to the AUR and Nix package registry are planned.

## Features

Rucola can be launched from the command line (`rucola`).
If no further arguments are given, rucola will open the notes in your default vault directory (specified in your config file).
This allows you to access your main vault of notes from anywhere in your file structure.
If no default vault is know yet, rucola will open in the current directory.
If you want to open a directory different from your default vault, you can pass the target as a positional argument, e.g. `rucola .` or `rucola ~/other/stuff`.

### Select Screen

Rucola initially launches into the *select screen*.
Here you will find an (unordered) list overview of all notes currently indexed by rucola, some statistics and a search bar.
The indexing finds only files with a select set of file extensions and respects your `.gitignore` file if present.
At the top, there are two blocks of statistics, referring to two environments:
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

You can configure which statistics block are shown at which time.

#### Filtering
The filtering works by a default fuzzy matcher.
In addition, you can include _condition words_ to search for notes fulfilling certain conditions.
A condition always starts with an identifier and goes to the next whitespace.
 - `#[tag]` declares a tag condition:
   Only notes with the given tag `[tag]` will be shown.
   Here you can use nested tags:
   A note tagged as `#math/topology` can be found by both the conditions `#math` and `#math/topology`, but not by `#topology` itself.
 - `!#[tag]` declares a tag exclusion condition:
   Only notes without the given tag `[tag]` will be shown.
 - `>[note]` declares a link condition:
   Only notes that contain a link to the note `[note]` will be shown.
   For notes whose name contains a whitespace, replace it with `-`.
 - `!>[note]` declares a link exclusion condition:
   Only notes that do not link to the note `[note]` will be shown.

Only words not starting with any of these identifiers will be used in the fuzzy match.
The order of these conditions words and fuzzy-matching words can be freely chosen.

For example, the filter string `#math !#math/topology >Topology map` shows all notes that are tagged as `#math` (or any nested subtag such as `#math/geometry`), not tagged with the nested tag `#math/topology` but still link to the `Topology` note and whose title contains some variation of `map`.

#### File Management
From the select view, you can access a couple of file management options for your notes:
 - Create a new note.
   If it has a valid note file extension (as defined in your config) and does not fall under the rules of your `.gitignore` file (if present), the note will then be added to the index.
 - Delete the selected note.
 - Rename the selected note.
   Your input cannot be a path, i.e. this cannot move the underlying file to a new location but only rename the existing file in its current location.
   You can however set a new extension, if no new extension is given the old extension will be reused.
   This will update links in other indexed files to point to the new location.
   If the new name has a non-note file extension (as defined in your config) or causes the file to fall under the rules of your `.gitignore` file (if present), the note will be removed from the index.
 - Move the selected to another location relative to your current vault path.
   If the input ends with a `/`, the path will be interpreted as a folder and the note, retaining its current name, will be moved into that folder.
   Otherwise, the input will be interpreted as a file location and the note moved there (and appropriately renamed to the last component of the path) while updating links in other indexed notes to point to the new location.
   In both cases, if there is no extension given and non-extension are not allowed per your config file, the original extension will be attached.
   If the move causes the note to end up with a non-note file extension (as defined in your config) or causes the file to fall under the rules of your `.gitignore` file (if present), the note will be removed from the index.
 - Edit the note in your configured text editor (such as a terminal based editor like vim or helix, or even obsidian).
   The used editor can be configured in the config file, if none is given, rucola defaults to your systems `$EDITOR` variable.
 - Reload the vault from the disk, in case you have made external changes.
   When opening a note as described in the previous point, rucola will automatically reload that (and only that note), so this feature should not be neccessary to often.
 - View the currently selected notes HTML representation in an external viewer.
   The selected note (and only the selected note) will be converted to HTML just in time for that purpose, so following HTML links may not always work.
 - Convert all currently filtered notes to HTML, so link following in the viewing feature above works as expected. Currently, you still need to do this manually whenever you have external changes to your files.


### Display Screen
The display screen shows a number of statistics about a single note:
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

Also, you can open the note in an external editor or view, converted to HTML in an external viewer.

### HTML conversion
Rucola can convert markdown notes to HTML documents, which are stored in the `.html` subfolder of your vault directory.
This feature uses [comrak](https://github.com/kivikakk/comrak) for the markdown-HTML conversion and supports most of the usual markdown syntax.
This is especially useful for notes that are difficult to read, for example because they contain lots of LaTeX code or tables - or simply because you prefer a more clean look. 
HTML files are automatically prepended with a `.css`-stylesheet reference if you have configured a source CSS-file, and with a MathJax-preamble if they contain LaTeX-blocks (with either `$...$` or `$$...$$`).
Also, you can perform small-scale string replacements in math mode, for example replacing `\field` with `\mathbb` to write fields more semantically clearly.

You can view a single HTML file from the select screen or the display screen, in this case it is converted just-in-time.
The file will be openend with the configured viewer (usually outside your terminal).
Alternatively, you can also convert all in the current local environment from the select view.
This allows you to follow links while viewing documents and is recommended.


### Configuration
Configuration files are - on Linux - stored in `XDG_CONFIGHOME/rucola`, which is usually `~/.config/rucola`.

Here is a list of all possible configuration settings:
 - `dynamic_filter` is set to `true` by default, but can be set to `false` to cause your select view to only filter upon pressing enter and not while typing.
 - `vault_path` is the path to your default vault that will be used by rucola unless overwritten by a command line positional argument.
 - `theme` is the name of the `.toml`-theme file to configure rucola's visual appearance.
 - `stats_show` is set to `Both` by default and configures which statistics blocks are shown at which time on the select screen.
   It can have one of three values:
   - `Both`: Both global and local stats are always shown.
   - `Local`: Only local stats are shown always, global stats are shown never.
     Note that you can still see the same values as global stats when using an empty filter.
   - `Relevant`: When you have no filter set, the global stats are shown.
     Otherwise, local stats are shown.
     This setting therefore avoids showing duplicating stats at all times.
 - `default_extension` is the extension appended to notes created by rucola, `.md` by default.
 - `file_types` lists all types of files to be indexed by rucola when opening a folder.
   Per default, this is set to `["markdown"]`, tracking files with the extions `.md`, `.markdown`, ...
   A full list of available file types can be found [here](https://docs.rs/ignore/latest/src/ignore/default_types.rs.html).
   It is currently not possible to define your own file types.
 - `editor` configures the command to edit your notes.
   This can be a terminal application or an external application.
 - `viewer` configures the command for your HTML viewing application (I use `google-chrome-stable`). If unconfigured, tries to use your systems default application for HTML files.
 - `mathjax` is set to `true` by default, but can be set to `false` to never prepend a MathJax preamble.
 - `math_replacments` is a vector of pairs of strings.
   In math mode, every appearance of the first string will be replaced by the second one.
   The default replaces `field` with `mathbb` and `lieagl` with `mathfrak` as an example for the TOML syntax and the general idea of using semantically valuable string replacements to make your LaTeX code clearer.
 - `css` is the name of your css style sheet (in your rucola config folder).
   The `.css` file extension can be omitted.
   If not set, no css file will be added to your HTML files.
 - `html_prepend` can contain any text you want to prepend to all your HTML files in addition to the mathjax, css and title tags/scripts.

## Planned Features
Planned features include:
 - More powerful search options, such as
   - Full text search through your files
 - Link updating on moving or renaming files
 - Hot reloading of notes and html conversions
 - Github-Wiki
 - Integration of Obisidan URI for opening notes in obsidian

## Technology
Rucola is implemented using the [ratatui](https://ratatui.rs) framework in [Rust](https://www.rust-lang.org/).

## License
Rucola is released under the [GNU General Public License v3](https://www.gnu.org/licenses/gpl-3.0).

Copyright (C) 2024 Linus Mußmächer <linus.mussmaecher@gmail.com>

This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with this program.  If not, see <https://www.gnu.org/licenses/>.
