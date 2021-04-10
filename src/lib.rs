//!
//! A library for convering between CESU-8 and UTF-8.
//!
//! # Examples
//!
//! > Unicode code points from the [Basic Multilingual Plane][bmp] (BMP), i.e. a
//! > code point in the range U+0000 to U+FFFF is encoded in the same way as
//! > UTF-8.
//!
//! [bmp]: https://en.wikipedia.org/wiki/Plane_(Unicode)#Basic_Multilingual_Plane
//!
//! If `cesu8::encode()` or `cesu8::decode()` only encounters data that is both
//! valid CESU-8 and UTF-8 data, the `cesu8` crate leverages this using a
//! [clone-on-write smart pointer][cow] ([Cow][rust-cow]). This means that there
//! are no unnecessary operations and needless allocation of memory:
//!
//! [cow]: https://en.wikipedia.org/wiki/Copy-on-write
//! [rust-cow]: https://doc.rust-lang.org/std/borrow/enum.Cow.html
//!
//! ```rust
//! use std::borrow::Cow;
//!
//! let str = "Hello, world!";
//! assert_eq!(cesu8::encode(str), Cow::Borrowed(str.as_bytes()));
//! assert_eq!(cesu8::decode(str.as_bytes()).unwrap(), Cow::Borrowed(str));
//! ```
//!
//! When data needs to be encoded or decoded, it functions as one might expect:
//!
//! ```
//! # use std::borrow::Cow;
//! let str = "\u{10401}";
//! let cesu8_data = &[0xED, 0xA0, 0x81, 0xED, 0xB0, 0x81];
//! assert_eq!(cesu8::decode(cesu8_data).unwrap(), Cow::Borrowed(str));
//! ```
//!
//! # Technical Details
//!
//! > The **Compatibility Encoding Scheme for UTF-16: 8-Bit (CESU-8)** is a
//! > variant of UTF-8 that is described in [Unicode Technical Report #26]
//! > [report]. A Unicode code point from the [Basic Multilingual Plane][bmp]
//! > (BMP), i.e. a code point in the range U+0000 to U+FFFF is encoded in the
//! > same way as UTF-8. A Unicode supplementary character, i.e. a code point in
//! > the range U+10000 to U+10FFFF, is first represented as a surrogate pair,
//! > like in [UTF-16][utf-16], and then each surrogate point is encoded in
//! > UTF-8. Therefore, CESU-8 needs six bytes (3 bytes per surrogate) for each
//! > Unicode supplementary character while UTF-8 needs only four. Though not
//! > specified in the technical report, *unpaired* surrogates are also encoded
//! > as 3 bytes each, and CESU-8 is exactly the same as applying an older
//! > [UCS-2] to UTF-8 converter to UTF-16 data.
//!
//! [report]: https://www.unicode.org/reports/tr26/tr26-4.html
//! [utf-16]: https://en.wikipedia.org/wiki/UTF-16
//! [ucs-2]: https://en.wikipedia.org/wiki/Universal_Coded_Character_Set
//!
//! > CESU-8 is not an official part of the Unicode Standard, because Unicode
//! > Technical Reports are informative documents only. It should be used
//! > exclusively for internal processing and never for external data exchange.
//!
//! ## Security
//!
//! As a general rule, this library is intended to fail on malformed or
//! unexpected input. This is desired, as CESU-8 should only be used for
//! internal use, any error should signify an issue with a developer's code or
//! some attacker is trying to improperly encode data to evade security checks.
//!
//! ## Surrogate Pairs and UTF-8
//!
//! The UTF-16 encoding uses "surrogate pairs" to represent Unicode code
//! points in the range from U+10000 to U+10FFFF.  These are 16-bit numbers
//! in the range 0xD800 to 0xDFFF.
//!
//! * 0xD800 to 0xDBFF: First half of surrogate pair.  When encoded as
//!   CESU-8, these become **1110**1101 **10**100000 **10**000000 to
//!   **1110**1101 **10**101111 **10**111111.
//!
//! * 0xDC00 to 0xDFFF: Second half of surrogate pair.  These become
//!   **1110**1101 **10**110000 **10**000000 to
//!   **1110**1101 **10**111111 **10**111111.
//!
//! Wikipedia [explains][utf-16] the
//! code point to UTF-16 conversion process:
//!
//! > Consider the encoding of U+10437 (𐐷):
//! >
//! > * Subtract 0x10000 from 0x10437. The result is 0x00437, 0000 0000 0100
//! >   0011 0111.
//! > * Split this into the high 10-bit value and the low 10-bit value:
//! >   0000000001 and 0000110111.
//! > * Add 0xD800 to the high value to form the high surrogate: 0xD800 +
//! >   0x0001 = 0xD801.
//! > * Add 0xDC00 to the low value to form the low surrogate: 0xDC00 +
//! >   0x0037 = 0xDC37.
//!
//! #  Related Work
//! This crate is a modified version of [Eric Kidd's](https://github.com/emk)
//! [`cesu-rs` repository](https://github.com/emk/cesu8-rs).
//! This crate was developed for [Residua](https://github.com/residua) as part
//! of their technical philosophy to have no external dependencies.

mod error;

use std::borrow::Cow;
use std::str::from_utf8;

pub use error::DecodingError;

/// Converts a slice of bytes to a string slice.
///
/// First, if the slice of bytes is already valid UTF-8, this function is
/// functionally no different than `std::str::from_utf8`; this means that
/// `decode()` does not need to perform any further operations and doesn't need
/// to allocate additional memory.
///
/// If the slice of bytes is not valid UTF-8, `decode()` works on the assumption
/// that the slice of bytes, if not valid UTF-8, is valid CESU-8. It will then
/// decode the bytes given to it and return the newly constructed string slice.
///
/// If the slice of bytes is found not to be valid CESU-8 data, `decode()`
/// returns `Err(DecodingError)` to signify that an error has occured.
///
/// ```
/// use std::borrow::Cow;
///
/// let str = "Hello, world!";
/// // Since 'str' is valid UTF-8 and CESU-8 data, 'cesu8::decode' can decode
/// // the string slice without allocating memory.
/// assert_eq!(cesu8::decode(str.as_bytes()).unwrap(), Cow::Borrowed(str));
///
/// let str = "\u{10401}";
/// let cesu8_data = &[0xED, 0xA0, 0x81, 0xED, 0xB0, 0x81];
/// // 'cesu8_data' is a byte slice containing a 6-byte surrogate pair which
/// // becomes a 4-byte UTF-8 character.
/// assert_eq!(cesu8::decode(cesu8_data).unwrap(), Cow::Borrowed(str));
/// ```
pub fn decode(bytes: &[u8]) -> Result<Cow<str>, DecodingError> {
    if let Ok(str) = from_utf8(bytes) {
        return Ok(Cow::Borrowed(str));
    }

    let mut decoded = Vec::with_capacity(bytes.len());
    let mut iter = bytes.iter();

    macro_rules! err {
        () => {
            return Err(DecodingError)
        };
    }

    macro_rules! next {
        () => {
            match iter.next() {
                Some(&byte) => byte,
                None => return Err(DecodingError),
            }
        };
    }

    macro_rules! next_continuation {
        () => {{
            let byte = next!();
            if is_continuation_byte(byte) {
                byte
            } else {
                return Err(DecodingError);
            }
        }};
    }

    while let Some(&first) = iter.next() {
        if first <= MAX_ASCII_CODE_POINT {
            decoded.push(first)
        } else {
            let width = match utf8_char_width(first) {
                Some(v) => v,
                None => err!(),
            };
            let second = next_continuation!();
            match width {
                2 => decoded.extend_from_slice(&[first, second]),
                3 => {
                    let third = next_continuation!();
                    match (first, second) {
                        (0xE0, 0xA0..=0xBF)
                        | (0xE1..=0xEC, 0x80..=0xBF)
                        | (0xED, 0x80..=0x9F)
                        | (0xEE..=0xEF, 0x80..=0xBF) => {
                            decoded.extend_from_slice(&[first, second, third]);
                        }
                        (0xED, 0xA0..=0xAF) => {
                            let fourth = next!();
                            if fourth != 0xED {
                                err!();
                            }
                            let fifth = next_continuation!();
                            if !(0xB0..=0xBF).contains(&fifth) {
                                err!();
                            }
                            let sixth = next_continuation!();
                            decoded.extend_from_slice(&decode_surrogate_pair(
                                second, third, fifth, sixth,
                            ));
                        }
                        _ => err!(),
                    }
                }
                _ => err!(),
            }
        }
    }

    debug_assert!(from_utf8(&decoded).is_ok());
    Ok(Cow::Owned(unsafe { String::from_utf8_unchecked(decoded) }))
}

#[inline]
fn decode_surrogate_pair(second: u8, third: u8, fifth: u8, sixth: u8) -> [u8; 4] {
    let surrogate1 = decode_surrogate(second, third);
    let surrogate2 = decode_surrogate(fifth, sixth);
    let code_point = 0x10000 + ((surrogate1 - 0xD800) << 10 | (surrogate2 - 0xDC00));
    decode_code_point(code_point)
}

#[inline]
fn decode_surrogate(second: u8, third: u8) -> u32 {
    const VAL_MASK: u8 = 0b0011_1111;
    0xD000 | ((second & VAL_MASK) as u32) << 6 | (third & VAL_MASK) as u32
}

#[inline]
fn decode_code_point(code_point: u32) -> [u8; 4] {
    const STRT_TAG: u8 = 0b1111_0000;
    [
        STRT_TAG | ((code_point & 0b1_1100_0000_0000_0000_0000) >> 18) as u8,
        CONT_TAG | ((code_point & 0b0_0011_1111_0000_0000_0000) >> 12) as u8,
        CONT_TAG | ((code_point & 0b0_0000_0000_1111_1100_0000) >> 6) as u8,
        CONT_TAG | ((code_point & 0b0_0000_0000_0000_0011_1111) as u8),
    ]
}

/// Converts a string slice to CESU-8 bytes.
///
/// If the string slice's representation in CESU-8 would be identical to its
/// present UTF-8 representation, this function is functionally no different
/// than `(&str).as_bytes()`; this means that `encode()` does not need to
/// perform any further operations and doesn't need to allocate any additional
/// memory.
///
/// If the string slice's representation in UTF-8 is not equivalent in CESU-8,
/// `encode()` encodes the string slice to its CESU-8 representation as a slice
/// of bytes.
///
/// ```
/// use std::borrow::Cow;
/// use cesu8;
///
/// let str = "Hello, world!";
/// // Since 'str' is valid UTF-8 and CESU-8 data, 'cesu8::encode' can encode
/// // data without allocating memory.
/// assert_eq!(cesu8::encode(str), Cow::Borrowed(str.as_bytes()));
///
/// let utf8_data = "\u{10401}";
/// let cesu8_data = Cow::Borrowed(&[0xED, 0xA0, 0x81, 0xED, 0xB0, 0x81]);
/// // 'utf8_data' is a 4-byte UTF-8 representation, which becomes a 6-byte
/// // CESU-8 representation.
/// assert_eq!(cesu8::encode(utf8_data), cesu8_data);
/// ```
pub fn encode(str: &str) -> Cow<[u8]> {
    if is_valid(str) {
        return Cow::Borrowed(str.as_bytes());
    }

    let bytes = str.as_bytes();
    let capacity = encoded_len(str);
    let mut encoded = Vec::with_capacity(capacity);
    let mut index = 0;

    while index < bytes.len() {
        let byte = bytes[index];
        if byte <= MAX_ASCII_CODE_POINT {
            encoded.push(byte);
            index += 1;
        } else {
            let width = utf8_char_width(byte).unwrap();
            let slice_range = index..index + width;
            if width <= CESU8_MAX_CHAR_WIDTH {
                encoded.extend(&bytes[slice_range])
            } else {
                let str = &str[slice_range];
                let code_point = str.chars().next().unwrap() as u32;
                let surrogate_pair = to_surrogate_pair(code_point);
                let encoded_pair = encode_surrogate_pair(surrogate_pair);
                encoded.extend(&encoded_pair)
            }
            index += width;
        }
    }

    Cow::Owned(encoded)
}

#[inline]
fn encode_surrogate_pair(surrogate_pair: [u16; 2]) -> [u8; 6] {
    let [b1, b2, b3] = encode_surrogate(surrogate_pair[0]);
    let [b4, b5, b6] = encode_surrogate(surrogate_pair[1]);
    [b1, b2, b3, b4, b5, b6]
}

#[inline]
fn encode_surrogate(surrogate: u16) -> [u8; 3] {
    const STRT_TAG: u8 = 0b1110_0000;
    [
        STRT_TAG | ((surrogate & 0b1111_0000_0000_0000) >> 12) as u8,
        CONT_TAG | ((surrogate & 0b0000_1111_1100_0000) >> 6) as u8,
        CONT_TAG | ((surrogate & 0b0000_0000_0011_1111) as u8),
    ]
}

#[inline]
fn to_surrogate_pair(code_point: u32) -> [u16; 2] {
    let code_point = code_point - 0x10000;
    let first = ((code_point >> 10) as u16) | 0xD800;
    let second = ((code_point & 0x3FF) as u16) | 0xDC00;
    [first, second]
}

/// Given a string slice, this function returns how many bytes in CESU-8 are
/// required to encode the string slice.
pub fn encoded_len(str: &str) -> usize {
    let bytes = str.as_bytes();
    let mut capacity = 0;
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte <= MAX_ASCII_CODE_POINT {
            capacity += 1;
            index += 1;
        } else {
            let width = utf8_char_width(byte).unwrap();
            capacity += if width <= CESU8_MAX_CHAR_WIDTH {
                width
            } else {
                6
            };
            index += width;
        }
    }
    capacity
}

/// Returns `true` if a string slice contains UTF-8 data that is also valid
/// CESU-8. This is mainly used in testing if a string slice needs to be
/// explicitly encoded using `cesu8::encode()`.
///
/// If `is_valid()` returns `false`, it implies that `&str.as_bytes()` is
/// directly equivalent to the string slice's CESU-8 representation.
///
/// ```
/// let str = "Hello, world!";
/// if cesu8::is_valid(&str) {
///     println!("str contains valid CESU-8 data")
/// } else {
///     panic!("str does not contain valid CESU-8 data")
/// }
///
/// // Any code point above U+10400 encoded in UTF-8 is not valid CESU-8.
/// assert!(!cesu8::is_valid("\u{10401}"));
/// ```
pub fn is_valid(str: &str) -> bool {
    for byte in str.bytes() {
        if is_continuation_byte(byte) {
            continue;
        }
        if let Some(width) = utf8_char_width(byte) {
            if width > CESU8_MAX_CHAR_WIDTH {
                return false;
            }
        } else {
            return false;
        }
    }
    true
}

/// The maximum UTF-8 character width in bytes that CESU-8 can use as-is.
const CESU8_MAX_CHAR_WIDTH: usize = 3;

#[inline]
fn is_continuation_byte(byte: u8) -> bool {
    const TAG_MASK: u8 = 0b1100_0000;
    byte & TAG_MASK == CONT_TAG
}

/// The prefix of a continuation byte in UTF-8 is **10**xxxxxx
const CONT_TAG: u8 = 0b1000_0000;

/// Given a byte that is the first byte of a UTF-8 character, `utf_char_width()`
/// returns the number of bytes the character uses to encode its code point in
/// the form of `Some(usize)` where the `usize` value is guaranteed to be in the
/// range [1, 4]. Otherwise, if the byte is not a valid first byte of a UTF-8
/// code point, `utf8_char_width()` returns `None`.
fn utf8_char_width(byte: u8) -> Option<usize> {
    match byte {
        0x00..=MAX_ASCII_CODE_POINT => Some(1),
        0xC2..=0xDF => Some(2),
        0xE0..=0xEF => Some(3),
        0xF0..=0xF4 => Some(4),
        _ => None,
    }
}

/// The last code point that ASCII can represent. Also, the last code point that
/// can be represented only with one byte.
const MAX_ASCII_CODE_POINT: u8 = 0x7F;
