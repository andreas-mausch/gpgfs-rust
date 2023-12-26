use std::error::Error;
use std::fs;
use std::fs::create_dir;
use std::path::Path;

use fuser::MountOption::{FSName, RW};
use pretty_assertions::assert_eq;

use gpgfs_rust::filesystem::HelloFS;

#[test]
fn test_list_directory_entries() -> Result<(), Box<dyn Error>> {
    use temp_dir::TempDir;
    let directory = TempDir::new()?;

    let plain = directory.path().join("plain");
    create_dir(&plain)?;

    let options = vec![RW, FSName("gpgfs-rust".to_string())];
    let session = fuser::spawn_mount2(HelloFS, &plain, &options)?;

    assert_eq!(get_files(&plain)?, vec!["hello.txt"]);

    session.join();
    Ok(())
}

fn get_files(path: &Path) -> Result<Vec<String>, Box<dyn Error>> {
    Ok(fs::read_dir(path)?
        .filter_map(|entry| Some(entry.ok()?.path().file_name()?.to_str()?.to_string()))
        .collect::<Vec<_>>())
}
