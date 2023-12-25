gpgfs-rust aims to be an alternative for gpgfs, which is written in Python (see below).

# Developer instructions

## Build

```bash
cargo build [--release]
```

## Run

```bash
cargo run 1111111190ABCDEF1234567890ABCDEF11111111 ./test-mount/
```

## Format

```bash
cargo fmt
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

- [https://github.com/jseppanen/gpgfs](gpgfs)
  This is basically a rewrite of gpgfs (which is Python) in Rust.
  Note the Merge Request for Python 3 support [here](https://github.com/jseppanen/gpgfs/pull/2)
- gocryptfs
  very good, fast and clean, but does not support GPG encryption
  you can use the `-extpass` cli option, to use for example a password from [https://www.passwordstore.org/](pass).
  However, it is not really the same as using GPG encryption. You won't be able to decrypt files with just your GPG key,
  you would need the password store as well.
- gpgtar
  Does not support mounting into the filesystem
- https://github.com/google/fuse-archive
  Does not support GPG
- VeraCrypt
  Does not support GPG
- [https://github.com/authenticationfailure/bhfs](BHFS - Black Hole Filesytem)
  Needs two tools
