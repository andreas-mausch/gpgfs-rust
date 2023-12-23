gpgfs-rust aims to be an alternative for gpgfs, which is written in Python (see below).

Alternatives:

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
