# `residua-cesu8` &nbsp;[![Build Status]][actions] [![Latest Version]][crates.io]

[Build Status]: https://img.shields.io/github/workflow/status/residua/cesu8/CI?logo=github
[actions]: https://github.com/residua/cesu8/actions/workflows/ci.yml
[Latest Version]: https://img.shields.io/crates/v/residua-cesu8?logo=rust
[crates.io]: https://crates.io/crates/residua-cesu8

*A library for converting between CESU-8 and UTF-8.*
View the documentation on `docs.rs` [here][docs].

[docs]: https://docs.rs/residua-cesu8

## Usage

This crate is [on crates.io][crates] and can be used by adding `residua-cesu8`
to your dependencies in your project's `Cargo.toml`:

```toml
[dependencies]
residua-cesu8 = "1"
```

[crates]: https://crates.io/crates/residua-cesu8

## Examples

Basic usage:

```rust
use std::borrow::Cow;
use cesu8::{from_cesu8, to_cesu8};

let str = "Hello, world!";
assert_eq!(to_cesu8(str), Cow::Borrowed(str.as_bytes()));
assert_eq!(from_cesu8(str.as_bytes()), Ok(Cow::Borrowed(str)));
```

When data needs to be encoded or decoded, it functions as one might expect:

```rust
use std::borrow::Cow;
use cesu8::from_cesu8;

let str = "\u{10400}";
let cesu8_data = &[0xED, 0xA0, 0x81, 0xED, 0xB0, 0x80];
let result: Result<Cow<str>, cesu8::DecodingError> = from_cesu8(cesu8_data);
assert_eq!(result.unwrap(), Cow::<str>::Owned(String::from(str)));
```

## License

Licensed under either of

-   Apache License, Version 2.0
    ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
-   MIT license
    ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
