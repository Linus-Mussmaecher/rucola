# Giraffe

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg?style=flat-square)](https://www.gnu.org/licenses/gpl-3.0)
![actions](https://img.shields.io/github/actions/workflow/status/Linus-Mussmaecher/giraffe/continuous-testing.yml?label=tests&style=flat-square)
![commits](https://img.shields.io/github/commit-activity/m/Linus-Mussmaecher/giraffe?style=flat-square)
[![tech1](https://img.shields.io/badge/-Rust-000000?logo=rust&style=flat-square)](https://www.rust-lang.org/)
[![tech2](https://img.shields.io/badge/-Ratatui-000000?logo=gnome-terminal&style=flat-square)](https://ratatui.rs)

Terminal-based browser and information aggregator for markdown file structures.

> [!CAUTION]
> This project is a work-in-progress by a single developer.
> Many Features are still lacking and bugs may appear frequently.
> All features described in the [features](#features) part are **target features** for version `1.0.0` and not neccessarily implemented yet.

## Contents
 - [Goals](#Goals)
 - [Installation](#installation)
 - [Feautures](#features)
  - [Overview Screen](#overview-screen)
  - [Single-Note Screen](#single-note-screen)
  - [Configuration](#configuration)
 - [Technology & License](#technology-license)

## Goals
 - *Target audience*: Users of a [zettelkasten-style](https://en.wikipedia.org/wiki/Zettelkasten) note system of interlinked markdown notes.
 - To present the user with high-level information & statistics about their entire note set.
 - To show the same information about filtered subsets of notes, as well as their relation with the entire note set.
 - To allow the user to view link and backlink as well as statistical information about a single note.
 - Allow the user to make small edits (such as renaming or changing tags) from within the application, and open the note in more sophisticated, user-specified editors and viewers.
 - Provide all of this functionality without leaving the terminal.

## Installation

## Features

### Overview Screen

### Single-Note Screen

### Configuration
Configuration files are - on Linux - stored in `XDG_CONFIGHOME/giraffe`, which is usually `~/.config/giraffe`.

## Technology & License
Giraffe is implemented using the [ratatui](https://ratatui.rs) framework in [Rust](https://www.rust-lang.org/) and released under the [GNU GPL v3 License](https://www.gnu.org/licenses/gpl-3.0).
