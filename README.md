gpgfs-rust aims to be an alternative for [gpgfs](https://github.com/jseppanen/gpgfs),
which is written in Python (see below).

It mounts a folder with encrypted files using [FUSE](https://en.wikipedia.org/wiki/Filesystem_in_Userspace)
and transparently en- and decrypts all files automatically using the given GPG key.

Performance is not a priority. I just need a tool to conveniently use GPG encrypted files.
I personally use it with [GnuCash](https://www.gnucash.org).

This project also serves as an exercise for me to gain more practice with Rust.

**Current status: Not even close to be functional.**

# Developer instructions

## Build

```bash
cargo build [--release]
```

## Run

```bash
cargo run 1111111190ABCDEF1234567890ABCDEF11111111 ./test-mount/
```

Replace `1111111190ABCDEF1234567890ABCDEF11111111` with your GPG key fingerprint.

## Format

```bash
cargo fmt
```

## Test

```bash
cargo test
```

## Check

```bash
cargo clippy -- --deny clippy::unwrap_used --deny clippy::expect_used --warn clippy::pedantic --deny warnings
```

## Find outdated dependencies

```bash
cargo outdated --root-deps-only
```

(This might require [cargo-outdated](https://archlinux.org/packages/extra/x86_64/cargo-outdated/))

# Alternatives

- [gpgfs](https://github.com/jseppanen/gpgfs)
  This is basically a rewrite of gpgfs (which is Python) in Rust.
  Note the Merge Request for Python 3 support [here](https://github.com/jseppanen/gpgfs/pull/2).
- [gocryptfs](https://nuetzlich.net/gocryptfs/)
  Very good, fast and clean, but does not support GPG encryption.
  You can use the `-extpass` cli option, to use for example a password from [pass](https://www.passwordstore.org/).
  However, it is not really the same as using GPG encryption. You won't be able to decrypt files with just your GPG key,
  you would need the password store as well.
- [gpgtar](https://www.gnupg.org/documentation/manuals/gnupg/gpgtar.html)
  Does not support mounting into the filesystem
- [fuse-archive](https://github.com/google/fuse-archive)
  Does not support GPG
- [VeraCrypt](https://veracrypt.fr)
  Does not support GPG
- [BHFS - Black Hole Filesytem](https://github.com/authenticationfailure/bhfs)
  Needs two tools for encryption and decryption

# Helpful links

- [FUSE filesystems](https://zsiciarz.github.io/24daysofrust/book/vol1/day15.html)
