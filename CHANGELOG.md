# Version 0.3.4 - Operating System Support
 - Rucola can now be installed via homebrew
 - Added a title to the select screen
 - Added the current rucola version to both screens
 - Added a build script that copies the 5 default configuration files in the configuration folder on install.
 - The display screen now only shows unique links and backlinks. The select screen continues to count links (not linking notes), so linking note A twice from note B will count as two outlinks for A, two inlinks for B, but now B will only show up once in the backlinks list of A and A will only show up once in the links list of B.
 - Pathes are now canonicalized before being sent to external commands or used to create a file watcher, solving some issues previously appearing on Windows where mixing forward and backwards slashes would cause problems.

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
