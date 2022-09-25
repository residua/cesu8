# cesu8

A simple library for converting between CESU-8 and UTF-8.

[![Build Status]][actions]
[![Latest Version]][crates.io]

[Build Status]: https://img.shields.io/github/workflow/status/residua/cesu8/ci?logo=github
[actions]: https://github.com/residua/cesu8/actions/workflows/ci.yml
[Latest Version]: https://img.shields.io/crates/v/residua-cesu8?logo=rust
[crates.io]: https://crates.io/crates/residua-cesu8

## Documentation

View the examples and documentation on `docs.rs` here: https://docs.rs/residua-cesu8.

## Usage

This crate is [on crates.io][crates.io] and can be used by adding `residua-cesu8`
to your dependencies in your project's `Cargo.toml`:

```toml
[dependencies]
residua-cesu8 = "2"
```

## Features

- `std` implements `std::error::Error` on `Error`. By default this feature is
  enabled.

## License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
