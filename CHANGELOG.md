# Version 0.8.1 - Sorted Index Caching
- Added an additional check that only saves the index when a later reload is also desired.
- Changed the caching of the index to be sorted alphabetically by key to work better with version control software.

# Version 0.8.0 - Index Caching & More
- Slightly changed the way markdown links are parsed to allow them to use file extensions.
- Fixed a bug that caused rucola to panic and crash when run on a git repo with detached head.
- Added the option to cache the rucola index in a file when the program is closed, and to reload it on start up. This comes with several caveats laid out in the wiki and default config and is disabled by default.
- Added confirmation dialogue when deleting notes
- Added more support for multi-word tags
- Added option to match tags by prefix instead of exactly


# Version 0.7.1 - Bug Fix
- Fixed an issue that caused YAML parsing to sometimes panic when no content was found.

# Version 0.7.0 - Sorting & Filtering
 - Added date of last modification as a sorting option.
 - Added shuffling as a sorting option.
 - Added the option to specify a default sorting direction and mode in the configuration file.
 - Filtering will now temporarily override your sorting option and direction, clearing the filter will reset to your default choice of sorting option and direction.
 - Deleting a note now always triggers a confirmation dialogue before actually deleting.
 - Users can now filter for multi-word tags by replacing spaces with dashes just like when searching for multi-word titles.
 - Users now have the option to find tags not only by exact match, but also by searching for a prefix of entered tags, i.e. entering `#diffg` will find notes tagged as `#diffgeo` when this option is active. This behaviour can be toggled in the program, and the default option can be set in the config file.
 - Fixed a bug that caused rucola to sometimes fail to open the editor specified in the `$EDITOR` environment variable when on Mac.
 - Fixed a bug that caused parts of the file to be interpreted as YAML frontmatter when using breaks.
 - Fixed a bug that caused YAML frontmatter to be part of the parsed HTML files.
 - Added information about viewers and viewing keys to the default config file.
 - Fixed an error that caused the number of links or backlinks to show at the bottom instead of the top of the tables in the display screen.

# Version 0.6.0 - YAML frontmatter
 - Added minor git integration.
   If rucola detects a git repo in its vault path, it will display
     - a `!` if there are untracked changes.
     - a `+` if there are uncommitted changes.
     - a `^` if the branch is ahead of the remote.
     - a `v` if the branch is behind the remote.
 - Added support for YAML frontmatter.
   - You can now specify a title in the frontmatter that will override the title inferred from the file name.
   The title added this way is only used for display, not for linking or other internal purposes.
   - You can now add tags in the frontmatter that will be added to the tags found in the text.
   Tags added this way support up to one level of nesting.
 - Updated dependency versions.
 - Added additional information on how to enter and exit the filter box in the help menu.
 - Updated default configuration to emphasize that the splitting of arguments for viewers and editors is mandatory.

# Version 0.5.0 - Markdown Viewing
 - Users can now choose between viewing files as markdown or HTML.
 - Fixed a bug that caused an error on launch when configuration file did not agree with current internal configuration struct. Contribution by GitHub user Morsicus.
 - Fixed a bug that caused new markdown notes created from within rucola to have an incorrect tag instead of a level 1 title. Contribution by GitHub user Morsicus.

# Version 0.4.1 - Bug Fix
 - Fixed a major issue that would cause rucola to freeze for a long time when notes were edited while html conversion was enabled.

# Version 0.4.0 - Feature Update
This update implements features requested in GitHub Issues since the first release.
 - The loading screen now displays the different stages of the initialization process.
 - The loading screen now shows a warning if rucola is run from the home directory.
 - Rucola now accepts links in the format `[Text](linked-note)`.
 - Updated versions of `crossterm`, `ratatui` and `tui-textarea` to latest releases.
 - Numerous small refactoring changes.

# Version 0.3.6 - Cargo Dist fixes
 - Fixed the automatic release creation.

# Version 0.3.5 - Build Script Fixes
 - Fixed the build script to fail without panicking.

# Version 0.3.4 - Operating System Support

## Features
 - Rucola can now be installed via homebrew
 - Added a title to the select screen
 - Added the current rucola version to both screens
 - Added a build script that copies the 5 default configuration files in the configuration folder on install, if possible and not yet present.
 - The display screen now only shows unique links and backlinks. The select screen continues to count links, not linking notes.
 - Paths shows in the display screen are now always canonicalized and absolute.

 ## Bugfixes
 - Fixed a bug where paths sent to external applications would contain a mix of `\` and `/` and thus sometimes not work correctly.
 - Fixed a bug where non-unix systems would not use vault path from command line argument or config file.
 - Fixed a bug where CSS would not display correctly when using Firefox and Windows.
 - Fixed a bug where FileEvents send by Windows would not be recognized.


# Version 0.3.3 - Fixing Release
Fixed problems with `cargo dist`, fully removed homebrew.

# Version 0.3.2 - Fixing Release
Removed homebrew installer, added a release to `crates.io`.

# Version 0.3.1 - Platforms
 - Updated the usage of the `notify` dependency to be platform-independent, allowing compilation on windows and mac.
 - Added conditional compilation to the usage of the `expanduser` dependency to allow compilation on windows.

# Version 0.3.0 - Initial Release
This version marks the initial binary release for rucola.

After a month-long personal testing phase in daily use, various small bug fixes and QoL changes have been implemented.

# Version 0.2.0
At this version, all initially planned features have been implemented.

# Version 0.1.0
Initial version for rucola.
