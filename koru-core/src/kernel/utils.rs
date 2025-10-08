use std::path::{Path, PathBuf};



/// Helper that validates if path exists
pub fn does_file_exist<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();
    path.is_file()
}


static POSSIBLE_CONFIG_LOCATIONS: [&str; 1] = ["koru/config.lua"];


/// Finds 
pub fn locate_config_path() -> Option<PathBuf> {
    for config in POSSIBLE_CONFIG_LOCATIONS.iter() {
        if does_file_exist(config) {
            return Some(PathBuf::from(config));
        }
    }
    None
}