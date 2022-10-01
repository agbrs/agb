use std::{env, path::PathBuf};

#[derive(Debug)]
pub struct FindRootDirectoryError;

pub fn find_agb_root_directory() -> Result<PathBuf, FindRootDirectoryError> {
    let mut current_path = env::current_dir().map_err(|_| FindRootDirectoryError)?;

    while !current_path.clone().join("justfile").exists() {
        current_path = current_path
            .parent()
            .ok_or(FindRootDirectoryError)?
            .to_owned();
    }

    Ok(current_path)
}

#[cfg(test)]
mod tests {
    use super::find_agb_root_directory;

    #[test]
    fn find_agb_root_directory_works() {
        find_agb_root_directory().unwrap();
    }
}
