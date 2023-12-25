use std::error::Error;
use std::ffi::OsStr;
use std::process::ExitCode;
use std::time::{Duration, UNIX_EPOCH};

use clap::Parser;
use env_logger::Env;
use fuser::{
    FileAttr, Filesystem, FileType, MountOption, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry,
    Request,
};
use gpgme::{Context, Protocol};
use libc::{EIO, ENOENT};
use log::info;
use MountOption::{AllowRoot, AutoUnmount, FSName, RW};
use Protocol::OpenPgp;

const TTL: Duration = Duration::from_secs(1); // 1 second

const HELLO_DIR_ATTR: FileAttr = FileAttr {
    ino: 1,
    size: 0,
    blocks: 0,
    atime: UNIX_EPOCH, // 1970-01-01 00:00:00
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: FileType::Directory,
    perm: 0o755,
    nlink: 2,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
    blksize: 512,
};

const HELLO_TXT_CONTENT: &str = "Hello World!\n";

const HELLO_TXT_ATTR: FileAttr = FileAttr {
    ino: 2,
    size: 13,
    blocks: 1,
    atime: UNIX_EPOCH, // 1970-01-01 00:00:00
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: FileType::RegularFile,
    perm: 0o644,
    nlink: 1,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
    blksize: 512,
};

struct HelloFS;

impl Filesystem for HelloFS {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        if parent == 1 && name.to_str() == Some("hello.txt") {
            reply.entry(&TTL, &HELLO_TXT_ATTR, 0);
        } else {
            reply.error(ENOENT);
        }
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        match ino {
            1 => reply.attr(&TTL, &HELLO_DIR_ATTR),
            2 => reply.attr(&TTL, &HELLO_TXT_ATTR),
            _ => reply.error(ENOENT),
        }
    }

    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        _size: u32,
        _flags: i32,
        _lock: Option<u64>,
        reply: ReplyData,
    ) {
        if ino == 2 {
            let Ok(size) = usize::try_from(offset) else {
                reply.error(EIO);
                return;
            };
            reply.data(&HELLO_TXT_CONTENT.as_bytes()[size..]);
        } else {
            reply.error(ENOENT);
        }
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        if ino != 1 {
            reply.error(ENOENT);
            return;
        }

        let entries = vec![
            (1, FileType::Directory, "."),
            (1, FileType::Directory, ".."),
            (2, FileType::RegularFile, "hello.txt"),
        ];

        let Ok(size) = usize::try_from(offset) else {
            reply.error(EIO);
            return;
        };

        for (i, entry) in entries.into_iter().enumerate().skip(size) {
            // i + 1 means the index of the next entry
            let Ok(next_entry) = i64::try_from(i + 1) else {
                reply.error(EIO);
                return;
            };
            if reply.add(entry.0, next_entry, entry.1, entry.2) {
                break;
            }
        }
        reply.ok();
    }
}

/// Mount folder with encrypted GPG files via FUSE
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The GPG key to use for the encryption
    #[arg(required = true)]
    gpg_key_fingerprint: String,

    /// Location of the target directory, where the plain files will be shown
    #[arg(required = true)]
    mount_point: String,

    /// Automatically unmount on process exit
    #[arg(short = 'u', long)]
    auto_unmount: bool,

    /// Allow root user to access filesystem
    #[arg(short = 'r', long)]
    allow_root: bool,
}

fn main() -> Result<ExitCode, Box<dyn Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let args = Args::parse();
    let mut options = vec![RW, FSName("gpgfs-rust".to_string())];
    if args.auto_unmount {
        options.push(AutoUnmount);
    }
    if args.allow_root {
        options.push(AllowRoot);
    }
    info!("Options: {:?}", &options);

    let mut context = Context::from_protocol(OpenPgp)?;
    let key = context.get_key(args.gpg_key_fingerprint)?;
    let user_id = key.user_ids().next().ok_or("No user id found")?;
    info!("User ID: {}", user_id);

    fuser::mount2(HelloFS, args.mount_point, &options)?;
    Ok(ExitCode::SUCCESS)
}
