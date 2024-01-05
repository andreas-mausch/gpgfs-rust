# Fuse crate

I have found these crates for Fuse filesystems:

- [fuse-rs](https://github.com/zargony/fuse-rs)
- [fuser](https://github.com/cberner/fuser)
- [fuse-mt](https://github.com/wfraser/fuse-mt)

fuse-rs hasn't been updated in the last 4 years, so I started with fuser.
I works well, but is very low-level.

I later found fuse-mt, which I want to switch to, because it makes the implementation for me
a bit easier and also has an higher abstraction level which makes it more Rust-like
(for example using Results as return value instead of writing into the reply parameter).

It is based on fuser, however it doesn't seem to use the latest version (with `mount2()`).
I don't care about it's multi-threaded functionality (it is still a bonus), and I might lose a bit of performance
by using filenames instead of inodes, but it looks very simple to implement which is exactly what I am looking for.

# Testing

From my experience, testing is a bit less fun than in other languages.
My main point of criticism are the assertions.

I found [pretty_assertions](https://github.com/rust-pretty-assertions/rust-pretty-assertions), which formats
the diffs nicely.

But I had some trouble to even compare Rust std types, like a vector.
I couldn't find a popular (fluent) assertion library, either.

There is [spectral](https://github.com/cfrancia/spectral), which hasn't been updated in 8 years.

And I found [fluent-asserter](https://github.com/dmoka/fluent-asserter)
and [assertor](https://github.com/google/assertor).

I decided to try assertor. I only has 49 stars on GitHub, but it is hosted by Google and the API looks natural.
For example it offers `contains_exactly()` and `contains_exactly_in_order()` for vectors.
