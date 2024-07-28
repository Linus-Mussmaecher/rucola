# 🌱 Rucola

[<img alt="github" src="https://img.shields.io/badge/github-Linus--Mussmaecher/rucola-8da0cb?style=for-the-badge&labelColor=555555&color=8da0cb&logo=github">](https://github.com/Linus-Mussmaecher/rucola)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg?labelColor=555555&style=for-the-badge&logo=gnu)](https://www.gnu.org/licenses/gpl-3.0)
[<img alt="rust" src="https://img.shields.io/badge/Rust-2021_Edition-ed9974?labelColor=555555&logo=rust&style=for-the-badge">](https://www.rust-lang.org/)

[<img alt="commit status" src="https://img.shields.io/github/commit-activity/m/Linus-Mussmaecher/rucola?labelColor=555555&color=66c2a5&style=for-the-badge">](https://github.com/Linus-Mussmaecher/rucola/commits/main)
[<img alt="test status" src="https://img.shields.io/github/actions/workflow/status/Linus-Mussmaecher/rucola/continuous-testing.yml?label=tests&branch=main&labelColor=555555&style=for-the-badge">](https://github.com/Linus-Mussmaecher/rucola/actions?query=branch%3Amain)
[<img alt="cratesio" src="https://img.shields.io/crates/v/rucola-notes.svg?labelColor=555555&color=417a5a&logo=linux-containers&style=for-the-badge">](https://crates.io/crates/rucola-notes)
[<img alt="arch-user-repository" src="https://img.shields.io/aur/version/rucola-notes.svg?labelColor=555555&color=1793d1&logo=arch-linux&style=for-the-badge">](https://aur.archlinux.org/packages/rucola-notes)

Terminal-based markdown note manager to view statistics, explore connections and launch editing and viewing applications.

## Contents
 - [Features](#features)
 - [Target Audience and Similar Programs](#target-audience-and-similar-programs)
 - [Future Features](#future-features)
 - [Installation](#installation)
   - [Usage](#usage)
 - [Technology](#technology)
 - [License](#license)

## Features
 - Present users of a zettelkasten-like note system of interlinked markdown files with high-level information & statistics about their notes.
 - Show the same information about filtered subsets of notes, as well as their relation with the entire note set.
 - Allow the user to view and follow links and backlinks of a single note to see connections within their note graph.
 - Allow the user to make small edits such as renaming or moving notes from within the application.
 - Provide easy access to a powerful, external text editor for editing notes.
 - Optionally compile notes to HTML-documents, including LaTeX compilation and code highlighting, on the fly and show them in an external viewer.
 - Provide all of this functionality from within a terminal user interface.

### Images

In the select & overview screen:

![select-screen](https://github.com/Linus-Mussmaecher/rucola/blob/main/readme-images/readme-image-select.png)

Viewing a single note:

![display-screen](https://github.com/Linus-Mussmaecher/rucola/blob/main/readme-images/readme-image-display.png)

A default light theme is also included.
> [!TIP]
> Default themes will adjust to your terminal colors, but the entire style can be fully customized.

## Target Audience and Similar Programs
Rucolas is made for users of a [zettelkasten-style](https://en.wikipedia.org/wiki/Zettelkasten) note system of interlinked markdown notes that want to do most of their note taking directly from within the terminal.

Compared to UI-note taking tools such as **Obsidian**, **Evernote** or **Notion** ...
 - rucola is much more light-weight, using no GUI, no web interface, no Electron, making it faster to open and more responsive to use, particularly on older hardware. Note that viewing your notes as HTML still requires a browser or other HTML viewer.
 - rucola provides easy access to edit notes in your favorite terminal text editor such as vim, emacs or helix, thus allowing a far more powerful editing interface. Although Obsidian provides vim keybindings via plugins, rucola allows you to edit in exactly the editing environment you are used to with all your vim plugins and other comfort settings ready to go.
 - rucola currently offers less extensibility and thus probably also less powerful statistics, in particular no graph view.
 - rucola relies on an external program to view HTML versions of your markdown notes, allowing once again more customizability (for example with the rich plugin system of browsers like firefox or chrome) as well as more conformity to your usual workflow.
 - rucola is not a single all-in-one application but works as a note browser that also glues an HTML viewer to an editor.

Compared to using only a terminal text editor (such as **vim**, **emacs** or **helix**) to manage markdown notes ...
 - rucola allows more note-specific interaction with your file system, such as following wiki links.
 - rucola provides note-specific statistics about your files, such as tags, nested tags and links statistics.
 - rucola facilitates more note-specific filtering and searching options based on links, tags, full text or titles.
 - rucola automatically renames wiki links on file rename, which the marksman language server currently does not support, making such a feature unavailable in helix and any other editor relying on LSP for markdown editing.
 - rucola allows you to view nicer-to-read HTML versions of your documents, complete with automatic background conversion (when enabled), which is especially useful for code that is difficult to read in the raw format. In particular, rucola compiles LaTeX and highlights code blocks.
 - rucola requires the usage of multiple programs in parallel instead of just your editor.

> [!TIP]
> Most text editors or terminal emulators provide some way to open a file in an already running instance - open rucola and your favourite editor next to one another and use rucola like a more note-specific file picker. 

## Future Features
The current version supports all features originally envisioned for this project and has been tested in daily use for some months.
This does not mean that development is finished.
Future plans include:
 - Frequent & quick bugfixes and similar non-breaking changes.
 - Incorporate user feedback into new features without losing sight of the original scope to also make rucola integrate into _your_ workflow.
 - As development on the underlying markdown parser [comrak](https://github.com/kivikakk/comrak) continues, especially support for wikilinks, certain internal algorithms can be made more safe & efficient (especially link updating on name changes).
   This is dependent on the progress on comrak.
 - As development on the [ratatui markdown reader](https://github.com/joshka/tui-markdown) continues, a possible future option would be to allow viewing of markdown files right within rucola.
   This is dependent on the progress on ratatui-markdown.

## Installation

Rucola has no requirements, but a [Nerd font](https://www.nerdfonts.com) may be required on your machine to display some of the icons properly.

#### Installation Scripts & Tarballs
For installation instructions via shell script, `.msi`, homebrew or tarball, see the [latest release](https://github.com/Linus-Mussmaecher/rucola/releases).
These installers are generated using [`cargo dist`](https://github.com/axodotdev/cargo-dist) and are provided for Windows, Mac and Linux.

#### AUR
Install `rucola` from the Arch User Repository:
```
 pacman -S rucola-notes
```
Install with an AUR helper such as `yay`:
```
 yay rucola
```
#### Crates.io
Install using `cargo` from [`crates.io`](https://crates.io/crates/rucola-notes):
```
 cargo install rucola-notes
```

#### Manual Build
If you want to build the latest commit of rucola by yourself, you can clone the repository, build & install: 
```
 git clone https://github.com/Linus-Mussmaecher/rucola
 cd rucola
 cargo install --path .
```

### Usage

Rucola can be launched from your command line with the `rucola` command.

> [!TIP]
> For more information on possible configuration options, features and usage tips, see the [GitHub Wiki](https://github.com/Linus-Mussmaecher/rucola/wiki). 

## Technology
Rucola is implemented using the [ratatui](https://ratatui.rs) framework in [Rust](https://www.rust-lang.org/). Markdown parsing is done via [comrak](https://github.com/kivikakk/comrak).

LaTeX compilation & macro system is facilitated by [KaTeX](https://katex.org/) while code highlighting is done with [highlight.js](https://highlightjs.org/).

## License
Rucola is released under the [GNU General Public License v3](https://www.gnu.org/licenses/gpl-3.0).

Copyright (C) 2024 Linus Mußmächer <linus.mussmaecher@gmail.com>

This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with this program.  If not, see <https://www.gnu.org/licenses/>.
