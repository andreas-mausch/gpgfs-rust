use std::ffi::OsStr;
use std::time::Duration;

use fuser::{Filesystem, ReplyDirectory, ReplyEntry, Request};
use libc::ENOSYS;
use log::debug;

const _TTL: Duration = Duration::from_secs(1); // 1 second

pub struct HelloFS;

impl Filesystem for HelloFS {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        debug!("lookup(parent: {parent:#x?}, name: {name:?})");
        reply.error(ENOSYS);
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        fh: u64,
        offset: i64,
        reply: ReplyDirectory,
    ) {
        debug!("readdir(ino: {ino:#x?}, fh: {fh}, offset: {offset})");
        reply.error(ENOSYS);
    }
}
