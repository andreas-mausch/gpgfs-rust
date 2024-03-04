use std::{io, mem};
use std::ffi::{CStr, CString, OsStr, OsString};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use FileType::{BlockDevice, CharDevice, Directory, NamedPipe, RegularFile, Socket, Symlink};
use fuse_mt::{DirectoryEntry, FileAttr, FilesystemMT, FileType, RequestInfo, ResultEmpty, ResultEntry, ResultOpen, ResultReaddir, ResultStatfs, ResultXattr, Statfs, Xattr};
use libc::EINVAL;
use log::{debug, error, warn};

use crate::libc_wrappers;

const TTL: Duration = Duration::from_secs(1); // 1 second

pub struct GpgFS {
    pub encrypted_directory: PathBuf,
}

impl GpgFS {
    fn real_path(&self, partial: &Path) -> OsString {
        self.encrypted_directory
            .join(partial.strip_prefix("/").unwrap())
            .into_os_string()
    }

    fn stat_real(&self, path: &Path) -> io::Result<FileAttr> {
        let real: OsString = self.real_path(path);
        debug!("stat_real: {:?}", real);

        match libc_wrappers::lstat(real) {
            Ok(stat) => {
                Ok(self.stat_to_fuse(stat))
            }
            Err(e) => {
                let err = io::Error::from_raw_os_error(e);
                error!("lstat({:?}): {}", path, err);
                Err(err)
            }
        }
    }

    fn stat_to_fuse(&self, stat: libc::stat64) -> FileAttr {
        // st_mode encodes both the kind and the permissions
        let kind = GpgFS::mode_to_filetype(stat.st_mode);
        let perm = (stat.st_mode & 0o7777) as u16;

        let time = |secs: i64, nanos: i64|
            SystemTime::UNIX_EPOCH + Duration::new(secs as u64, nanos as u32);

        // libc::nlink_t is wildly different sizes on different platforms:
        // linux amd64: u64
        // linux x86:   u32
        // macOS amd64: u16
        #[allow(clippy::cast_lossless)]
            let nlink = stat.st_nlink as u32;

        FileAttr {
            size: stat.st_size as u64,
            blocks: stat.st_blocks as u64,
            atime: time(stat.st_atime, stat.st_atime_nsec),
            mtime: time(stat.st_mtime, stat.st_mtime_nsec),
            ctime: time(stat.st_ctime, stat.st_ctime_nsec),
            crtime: SystemTime::UNIX_EPOCH,
            kind,
            perm,
            nlink,
            uid: stat.st_uid,
            gid: stat.st_gid,
            rdev: stat.st_rdev as u32,
            flags: 0,
        }
    }

    fn mode_to_filetype(mode: libc::mode_t) -> FileType {
        match mode & libc::S_IFMT {
            libc::S_IFDIR => Directory,
            libc::S_IFREG => RegularFile,
            libc::S_IFLNK => Symlink,
            libc::S_IFBLK => BlockDevice,
            libc::S_IFCHR => CharDevice,
            libc::S_IFIFO => NamedPipe,
            libc::S_IFSOCK => Socket,
            _ => { panic!("unknown file type"); }
        }
    }
}

impl FilesystemMT for GpgFS {
    fn getattr(&self, _req: RequestInfo, path: &Path, fh: Option<u64>) -> ResultEntry {
        debug!("getattr(path: {path:?})");

        if let Some(fh) = fh {
            match libc_wrappers::fstat(fh) {
                Ok(stat) => Ok((TTL, self.stat_to_fuse(stat))),
                Err(e) => Err(e)
            }
        } else {
            match self.stat_real(path) {
                Ok(attr) => Ok((TTL, attr)),
                Err(e) => Err(e.raw_os_error().unwrap())
            }
        }
    }

    fn getxattr(&self, _req: RequestInfo, path: &Path, name: &OsStr, size: u32) -> ResultXattr {
        debug!("getxattr: {:?} {:?} {}", path, name, size);

        let real = self.real_path(path);

        if size > 0 {
            let mut data = Vec::<u8>::with_capacity(size as usize);
            let nread = libc_wrappers::lgetxattr(
                real, name.to_owned(), unsafe { mem::transmute(data.spare_capacity_mut()) })?;
            unsafe { data.set_len(nread) };
            Ok(Xattr::Data(data))
        } else {
            let nbytes = libc_wrappers::lgetxattr(real, name.to_owned(), &mut [])?;
            Ok(Xattr::Size(nbytes as u32))
        }
    }

    fn opendir(&self, _req: RequestInfo, path: &Path, _flags: u32) -> ResultOpen {
        let real = self.real_path(path);
        debug!("opendir: {:?} (flags = {:#o})", real, _flags);
        match libc_wrappers::opendir(real) {
            Ok(fh) => Ok((fh, 0)),
            Err(e) => {
                let ioerr = io::Error::from_raw_os_error(e);
                error!("opendir({:?}): {}", path, ioerr);
                Err(e)
            }
        }
    }

    fn releasedir(&self, _req: RequestInfo, path: &Path, fh: u64, _flags: u32) -> ResultEmpty {
        debug!("releasedir: {:?}", path);
        libc_wrappers::closedir(fh)
    }

    fn readdir(&self, _req: RequestInfo, path: &Path, fh: u64) -> ResultReaddir {
        debug!("readdir(path: {path:?})");

        if fh == 0 {
            error!("readdir: missing fh");
            return Err(EINVAL);
        }

        let mut entries: Vec<DirectoryEntry> = vec![];

        loop {
            match libc_wrappers::readdir(fh) {
                Ok(Some(entry)) => {
                    let name_c = unsafe { CStr::from_ptr(entry.d_name.as_ptr()) };
                    let name = OsStr::from_bytes(name_c.to_bytes()).to_owned();

                    let filetype = match entry.d_type {
                        libc::DT_DIR => FileType::Directory,
                        libc::DT_REG => FileType::RegularFile,
                        libc::DT_LNK => FileType::Symlink,
                        libc::DT_BLK => FileType::BlockDevice,
                        libc::DT_CHR => FileType::CharDevice,
                        libc::DT_FIFO => FileType::NamedPipe,
                        libc::DT_SOCK => {
                            warn!("FUSE doesn't support Socket file type; translating to NamedPipe instead.");
                            FileType::NamedPipe
                        }
                        _ => {
                            let entry_path = PathBuf::from(path).join(&name);
                            let real_path = self.real_path(&entry_path);
                            match libc_wrappers::lstat(real_path) {
                                Ok(stat64) => GpgFS::mode_to_filetype(stat64.st_mode),
                                Err(errno) => {
                                    let ioerr = io::Error::from_raw_os_error(errno);
                                    panic!("lstat failed after readdir_r gave no file type for {:?}: {}",
                                           entry_path, ioerr);
                                }
                            }
                        }
                    };

                    entries.push(DirectoryEntry {
                        name,
                        kind: filetype,
                    })
                }
                Ok(None) => { break; }
                Err(e) => {
                    error!("readdir: {:?}: {}", path, e);
                    return Err(e);
                }
            }
        }

        Ok(entries)
    }

    fn statfs(&self, _req: RequestInfo, path: &Path) -> ResultStatfs {
        debug!("statfs: {:?}", path);

        let real = self.real_path(path);
        let mut buf: libc::statfs = unsafe { ::std::mem::zeroed() };
        let result = unsafe {
            let path_c = CString::from_vec_unchecked(real.into_vec());
            libc::statfs(path_c.as_ptr(), &mut buf)
        };

        if -1 == result {
            let e = io::Error::last_os_error();
            error!("statfs({:?}): {}", path, e);
            Err(e.raw_os_error().unwrap())
        } else {
            Ok(statfs_to_fuse(buf))
        }
    }
}

fn statfs_to_fuse(statfs: libc::statfs) -> Statfs {
    Statfs {
        blocks: statfs.f_blocks,
        bfree: statfs.f_bfree,
        bavail: statfs.f_bavail,
        files: statfs.f_files,
        ffree: statfs.f_ffree,
        bsize: statfs.f_bsize as u32,
        namelen: statfs.f_namelen as u32,
        frsize: statfs.f_frsize as u32,
    }
}
