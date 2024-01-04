use std::error::Error;
use std::fs::canonicalize;
use std::path::PathBuf;

use clap::{arg, Parser};
use env_logger::Env;
use fuser::MountOption;
use gpgme::{Context, Protocol};
use log::info;
use MountOption::{AllowRoot, AutoUnmount, FSName, RW};
use Protocol::OpenPgp;

use crate::filesystem::GpgFS;

mod filesystem;

/// Mount folder with encrypted GPG files via FUSE
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The GPG key to use for the encryption
    #[arg(required = true)]
    gpg_key_fingerprint: String,

    /// Location of the source directory, where the encrypted files are in
    #[arg(required = true)]
    encrypted_directory: PathBuf,

    /// Location of the target directory, where the plain files will be shown
    #[arg(required = true)]
    mount_point: PathBuf,

    /// Automatically unmount on process exit
    #[arg(short = 'u', long)]
    auto_unmount: bool,

    /// Allow root user to access filesystem
    #[arg(short = 'r', long)]
    allow_root: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let args = Args::parse();
    let mut options = vec![RW, FSName("gpgfs-rust".to_string())];
    if args.auto_unmount {
        options.push(AutoUnmount);
    }
    if args.allow_root {
        options.push(AllowRoot);
    }
    let encrypted_directory = canonicalize(args.encrypted_directory)?;
    let mount_point = canonicalize(args.mount_point)?;
    info!("Encrypted directory: {encrypted_directory:?}, Mount point: {mount_point:?}, Options: {options:?}");

    let mut context = Context::from_protocol(OpenPgp)?;
    let key = context.get_key(args.gpg_key_fingerprint)
        .map_err(|e| format!("GPG Key fingerprint not found: {e}"))?;
    let user_id = key.user_ids().next().ok_or("No user id found")?;
    info!("User ID: {user_id}");

    fuser::mount2(GpgFS { encrypted_directory }, mount_point, &options)?;
    Ok(())
}
