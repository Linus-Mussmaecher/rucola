use std::str::FromStr;

/// This build script checks for the presence of a configuration file and two default styles and creates them if not present.
fn main() {
    // Step 1: Get the supposed location for the config file
    if let Ok(target) = confy::get_configuration_file_path("rucola", "config") {
        // create it if not present and fill it with the default from the repo
        copy_with_create_if_not_present(
            std::path::PathBuf::from_str("./default-config/config.toml").unwrap(),
            target,
        );
    }

    // Step 2: Get the supposed location for the toml style file
    if let Ok(target) = confy::get_configuration_file_path("rucola", "default_dark") {
        // The default location for the css file is just an extension change away
        let css_target = {
            let mut css_target_temp = target.clone();
            css_target_temp.set_extension("css");
            css_target_temp
        };

        // create both if not present
        copy_with_create_if_not_present(
            std::path::PathBuf::from_str("./default-config/default_dark.toml").unwrap(),
            target,
        );

        copy_with_create_if_not_present(
            std::path::PathBuf::from_str("./default-config/default_dark.css").unwrap(),
            css_target,
        );
    }

    // Repeat this process for the light style
    if let Ok(target) = confy::get_configuration_file_path("rucola", "default_light") {
        // The default location for the css file is just an extension change away
        let css_target = {
            let mut css_target_temp = target.clone();
            css_target_temp.set_extension("css");
            css_target_temp
        };

        // create both if not present
        copy_with_create_if_not_present(
            std::path::PathBuf::from_str("./default-config/default_light.toml").unwrap(),
            target,
        );

        copy_with_create_if_not_present(
            std::path::PathBuf::from_str("./default-config/default_light.css").unwrap(),
            css_target,
        );
    }
}

/// Checks if the superfolder of the file at `target` is present.
/// If not, creates it.
/// Then checks if the file at `target` itself is present.
/// If not, creates it and copies the contents from `source` into it.
fn copy_with_create_if_not_present(source: std::path::PathBuf, target: std::path::PathBuf) {
    // check if parent exists
    if let Some(parent) = target.parent() {
        if !parent.exists() {
            // if not, create it
            std::fs::create_dir(parent).unwrap();
        }
    }

    // check if actual file exists
    if !target.exists() {
        // if not, create it
        let target_file = std::fs::File::create(&target);
        if target_file.is_ok() {
            // and copy a default
            std::fs::copy(&source, &target).unwrap();
        }
    }
}
