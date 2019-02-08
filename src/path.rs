use std::path::{Path, PathBuf};
use std::error::Error;

pub fn get_path() -> Result<PathBuf, Box<dyn Error>> {
    let path = Path::new(&std::env::var("HOME")?)
            .to_path_buf()
            .join(".local")
            .join("share")
            .join("zoo");
    std::fs::create_dir_all(&path)?;
    Ok(path)
}
