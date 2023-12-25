use std::error::Error;

use clap::Parser;
use env_logger::Env;
use fuser::MountOption;
use gpgme::{Context, Protocol};
use log::info;
use MountOption::{AllowRoot, AutoUnmount, FSName, RW};
use Protocol::OpenPgp;

use crate::filesystem::HelloFS;

mod filesystem;

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
    info!("Options: {options:?}");

    let mut context = Context::from_protocol(OpenPgp)?;
    let key = context.get_key(args.gpg_key_fingerprint)
        .map_err(|e| format!("GPG Key fingerprint not found: {e}"))?;
    let user_id = key.user_ids().next().ok_or("No user id found")?;
    info!("User ID: {user_id}");

    fuser::mount2(HelloFS, args.mount_point, &options)?;
    Ok(())
}
