# Version 0.5.0 - Markdown Viewing
 - Users can now choose between viewing files as markdown or HTML.
 - Fixed a bug that caused an error on launch when configuration file did not agree with current internal configuration struct. Contribution by GitHub user Morsicus.

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
