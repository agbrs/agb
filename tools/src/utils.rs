use std::{
    env,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct FindRootDirectoryError;

pub fn find_agb_root_directory() -> Result<PathBuf, FindRootDirectoryError> {
    let current_path = env::current_dir().map_err(|_| FindRootDirectoryError)?;

    let mut search_path: &Path = &current_path;

    while !search_path.join("justfile").exists() {
        search_path = search_path.parent().ok_or(FindRootDirectoryError)?;
    }

    Ok(search_path.to_owned())
}

#[cfg(test)]
mod tests {
    use super::find_agb_root_directory;

    #[test]
    fn find_agb_root_directory_works() {
        let agb_root = find_agb_root_directory().unwrap();
        assert!(agb_root.join("justfile").exists());
    }
}
