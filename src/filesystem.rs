extern crate fuser;

use std::ffi::{OsStr, OsString};
use std::os::unix::fs::DirEntryExt;
use std::path::PathBuf;
use std::time::{Duration, UNIX_EPOCH};

use fuser::{FileAttr, Filesystem, FileType, FUSE_ROOT_ID, ReplyAttr, ReplyDirectory, ReplyEntry, Request};
use fuser::FileType::{Directory, RegularFile};
use libc::ENOENT;
use log::{debug, error};

const TTL: Duration = Duration::from_secs(1); // 1 second

pub struct GpgFS {
    pub encrypted_directory: PathBuf,
}

trait GpgSimpleFS {
    fn file_list(&self) -> Result<Vec<DirectoryEntry>, std::io::Error>;
}

/// A directory entry.
#[derive(Clone, Debug)]
pub struct DirectoryEntry {
    /// Name of the entry
    pub name: OsString,
    /// Kind of file (directory, file, pipe, etc.)
    pub kind: FileType,
}

const HELLO_DIR_ATTR: FileAttr = FileAttr {
    ino: 1,
    size: 0,
    blocks: 0,
    atime: UNIX_EPOCH, // 1970-01-01 00:00:00
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: Directory,
    perm: 0o755,
    nlink: 2,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
    blksize: 512,
};

const _HELLO_TXT_ATTR: FileAttr = FileAttr {
    ino: 2,
    size: 13,
    blocks: 1,
    atime: UNIX_EPOCH, // 1970-01-01 00:00:00
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: RegularFile,
    perm: 0o644,
    nlink: 1,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
    blksize: 512,
};

impl GpgSimpleFS for GpgFS {
    fn file_list(&self) -> Result<Vec<DirectoryEntry>, std::io::Error> {
        Ok(std::fs::read_dir(&self.encrypted_directory)?
            .filter_map(|entry| Some(entry.ok()?)).enumerate()
            .map(|(index, entry)| {
                debug!("entry found: {index} {:?}, {:?}", entry.ino(), entry.file_name());
                DirectoryEntry { name: entry.file_name(), kind: to_file_type(entry.file_type().unwrap()) }
            })
            .collect())
    }
}

impl Filesystem for GpgFS {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        debug!("lookup(parent: {parent:#x?}, name: {name:?})");
        reply.error(ENOENT);
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        debug!("getattr(ino: {ino:#x?})");
        match ino {
            FUSE_ROOT_ID => reply.attr(&TTL, &HELLO_DIR_ATTR),
            _ => reply.error(ENOENT),
        }
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        debug!("readdir(ino: {ino:#x?}, fh: {fh}, offset: {offset})");

        if ino != FUSE_ROOT_ID {
            reply.error(ENOENT);
            return;
        }

        let Ok(file_list) = &self.file_list() else
        {
            error!("file_list() returned an error");
            reply.error(ENOENT);
            return;
        };

        file_list.iter().skip(offset as usize).enumerate()
            .for_each(|(index, entry)| {
                if !reply.add(FUSE_ROOT_ID + 1, offset + index as i64 + 1, entry.kind, entry.name.as_os_str()) {
                    error!("reply.add() failed for {index} {:?}", entry.name);
                }
            });

        reply.ok();
    }
}

fn to_file_type(file_type: std::fs::FileType) -> FileType {
    if file_type.is_dir() { Directory } else if file_type.is_file() { RegularFile } else {
        panic!("File type unknown.")
    }
}
