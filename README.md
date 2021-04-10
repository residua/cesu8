# CESU-8 &nbsp;[![Build Status]][actions] [![Latest Version]][crates.io]

[Build Status]: https://img.shields.io/github/workflow/status/residua/cesu8/CI?logo=github
[actions]: https://github.com/residua/cesu8/actions/workflows/ci.yml
[Latest Version]: https://img.shields.io/crates/v/residua-cesu8?logo=rust
[crates.io]: https://crates.io/crates/residua-cesu8

*A library for converting between CESU-8 and UTF-8.*

## Usage

> Unicode code points from the [Basic Multilingual Plane][bmp] (BMP), i.e. a
> code point in the range U+0000 to U+FFFF is encoded in the same way as
> UTF-8.

[bmp]: https://en.wikipedia.org/wiki/Plane_(Unicode)#Basic_Multilingual_Plane

If `cesu8::encode()` or `cesu8::decode()` only encounters data that is both valid CESU-8 and UTF-8
data, the `cesu8`
crate leverages this using a
[clone-on-write smart pointer][cow] ([Cow][rust-cow]). This means that there are no unnecessary
operations and needless allocation of memory:

[cow]: https://en.wikipedia.org/wiki/Copy-on-write
[rust-cow]: https://doc.rust-lang.org/std/borrow/enum.Cow.html

```rust
use std::borrow::Cow;

let str = "Hello, world!";
assert_eq!(cesu8::encode(STR), Cow::Borrowed(STR.as_bytes()));
assert_eq!(cesu8::decode(STR.as_bytes()).unwrap(), Cow::Borrowed(STR));
```

When data needs to be encoded or decoded, it functions as one might expect:

```rust
let str = "\u{10401}";
let cesu8_data = &[0xED, 0xA0, 0x81, 0xED, 0xB0, 0x81];
assert_eq!(cesu8::decode(cesu8_data).unwrap(), Cow::Borrowed(str));
```

## Technical Details

> The **Compatibility Encoding Scheme for UTF-16: 8-Bit (CESU-8)** is a variant of UTF-8 that is
> described in [Unicode Technical Report #26][report]. A Unicode code point from the
> [Basic Multilingual Plane][bmp] (BMP), i.e. a code point in the range U+0000 to U+FFFF is encoded
> in the  same way as UTF-8. A Unicode supplementary character, i.e. a code point in  the range
> U+10000 to U+10FFFF, is first represented as a surrogate pair, like in [UTF-16][utf-16], and then
> each surrogate point is encoded in  UTF-8. Therefore, CESU-8 needs six bytes (3 bytes per
> surrogate) for each  Unicode supplementary character while UTF-8 needs only four. Though not
> specified in the technical report, *unpaired* surrogates are also encoded as 3 bytes each, and
> CESU-8 is exactly the same as applying an older [UCS-2] to UTF-8 converter to UTF-16 data.

[report]: https://www.unicode.org/reports/tr26/tr26-4.html
[utf-16]: https://en.wikipedia.org/wiki/UTF-16
[ucs-2]: https://en.wikipedia.org/wiki/Universal_Coded_Character_Set

> CESU-8 is not an official part of the Unicode Standard, because Unicode Technical Reports are
> informative documents only. It should be used exclusively for internal processing and never for
> external data exchange.

### Security

As a general rule, this library is intended to fail on malformed or unexpected input. This is
desired, as CESU-8 should only be used for internal use, any error should signify an issue with a
developer's code or some attacker is trying to improperly encode data to evade security checks.

### Surrogate Pairs and UTF-8

The UTF-16 encoding uses "surrogate pairs" to represent Unicode code points in the range from
U+10000 to U+10FFFF. These are 16-bit numbers in the range 0xD800 to 0xDFFF.

* 0xD800 to 0xDBFF: First half of surrogate pair. When encoded as CESU-8, these become **1110**
  1101 **10**100000 **10**
  000000 to
  **1110**1101 **10**101111 **10**111111.
* 0xDC00 to 0xDFFF: Second half of surrogate pair. These become **1110**1101 **10**110000 **10**
  000000 to **1110**
  1101 **10**111111 **10**111111.

Wikipedia [explains][utf-16] the code point to UTF-16 conversion process:

> Consider the encoding of U+10437 (𐐷):
> * Subtract 0x10000 from 0x10437. The result is 0x00437, 0000 0000 0100 0011 0111.
> * Split this into the high 10-bit value and the low 10-bit value: 0000000001 and 0000110111.
> * Add 0xD800 to the high value to form the high surrogate: 0xD800 + 0x0001 = 0xD801.
> * Add 0xDC00 to the low value to form the low surrogate: 0xDC00 + 0x0037 = 0xDC37.

# Related Work

This crate is a modified version of [Eric Kidd's](https://github.com/emk)
[`cesu-rs` repository](https://github.com/emk/cesu8-rs). This crate was developed
for [Residua](https://github.com/residua) as part of their technical philosophy to have no external
dependencies

## License

This crate is available as open source under the terms of
the [MIT License](https://github.com/residua/cesu8/blob/latest/LICENSE.md).
