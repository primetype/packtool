# packtool

[![Continuous Health Check](https://github.com/primetype/packtool/actions/workflows/ci_check.yml/badge.svg?branch=master)](https://github.com/primetype/packtool/actions/workflows/ci_check.yml)

Rust's tooling to write packed objects. Objects that can be packed
into specific serialization format of fixed size.

Support rust from `1.51.0` onward.

# Example

a very simplified implementation of the TAR archive file format is implemented in
the [examples](examples) directory.

```
$ tar -cf example.tar README.md Cargo.toml
$ cargo run --example tar
compressed file: README.md (967 bytes)
compressed file: Cargo.toml (885 bytes)
```

## License

This project is licensed under the [MIT] **OR** [Apache-2.0] dual license.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in `keynesis` by you, shall be licensed as `MIT OR Apache-2.0` dual
license, without any additional terms or conditions.