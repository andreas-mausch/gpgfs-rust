use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::fs::{create_dir, write};
use std::path::Path;

use assertor::{assert_that, VecAssertion};
use fuse_mt::FuseMT;

use gpgfs_rust::filesystem::GpgFS;

#[test]
fn test_list_directory_entries() -> Result<(), Box<dyn Error>> {
    use temp_dir::TempDir;
    let directory = TempDir::new()?;

    let encrypted = directory.path().join("encrypted");
    create_dir(&encrypted)?;
    create_dir(encrypted.join("sub-dir"))?;
    write(encrypted.join("example-file.txt"), "This is some existing content.")?;

    let plain = directory.path().join("plain");
    create_dir(&plain)?;

    let options = [OsStr::new("fsname=gpgfs-rust")];
    let session = fuse_mt::spawn_mount(FuseMT::new(GpgFS { encrypted_directory: encrypted }, 1),
                                       &plain,
                                       &options[..])?;

    assert_that!(get_files(&plain)?)
        .contains_exactly(vec![
            "sub-dir".into(),
            "example-file.txt".into(),
        ]);

    session.join();
    Ok(())
}

fn get_files(path: &Path) -> Result<Vec<String>, std::io::Error> {
    Ok(fs::read_dir(path)?
        .filter_map(|entry| Some(entry.ok()?.path().file_name()?.to_str()?.to_string()))
        .collect::<Vec<_>>())
}
