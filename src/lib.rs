//! A library for converting between CESU-8 and UTF-8.
//!
//! > Unicode code points from the [Basic Multilingual Plane][bmp] (BMP), i.e. a
//! > code point in the range U+0000 to U+FFFF is encoded in the same way as
//! > UTF-8.
//!
//! [bmp]: https://en.wikipedia.org/wiki/Plane_(Unicode)#Basic_Multilingual_Plane
//!
//! If [`from_cesu8`] or [`to_cesu8`] only encounters data that is both
//! valid CESU-8 and UTF-8 data, the `cesu8` crate leverages this using a
//! [clone-on-write smart pointer][cow] ([`Cow`][rust-cow]). This means that there
//! are no unnecessary operations and needless allocation of memory:
//!
//! [cow]: https://en.wikipedia.org/wiki/Copy-on-write
//! [rust-cow]: https://doc.rust-lang.org/std/borrow/enum.Cow.html
//!
//! # Examples
//!
//! Basic usage:
//!
//! ```rust
//! use std::borrow::Cow;
//! use cesu8::{from_cesu8, to_cesu8};
//!
//! let str = "Hello, world!";
//! assert_eq!(to_cesu8(str), Cow::Borrowed(str.as_bytes()));
//! assert_eq!(from_cesu8(str.as_bytes()), Ok(Cow::Borrowed(str)));
//! ```
//!
//! When data needs to be encoded or decoded, it functions as one might expect:
//!
//! ```
//! # use std::borrow::Cow;
//! # use cesu8::from_cesu8;
//!
//! let str = "\u{10400}";
//! let cesu8_data = &[0xED, 0xA0, 0x81, 0xED, 0xB0, 0x80];
//! let result: Result<Cow<str>, cesu8::DecodingError> = from_cesu8(cesu8_data);
//! assert_eq!(result.unwrap(), Cow::<str>::Owned(String::from(str)));
//! ```

#![deny(clippy::pedantic)]
#![allow(clippy::cast_lossless, clippy::cast_possible_truncation)]

use std::{borrow::Cow, error::Error, fmt, str::from_utf8};

/// Converts a slice of bytes to a string slice.
///
/// First, if the slice of bytes is already valid UTF-8, this function is
/// functionally no different than [`std::str::from_utf8`](std::str::from_utf8);
/// this means that `from_cesu8()` does not need to perform any further
/// operations and doesn't need to allocate additional memory.
///
/// If the slice of bytes is not valid UTF-8, `from_cesu8()` works on the
/// assumption that the slice of bytes, if not valid UTF-8, is valid CESU-8.
/// It will then decode the bytes given to it and return the newly constructed
/// string slice.
///
/// # Errors
///
/// Returns [`DecodingError`] if the input is invalid CESU-8 data.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use std::borrow::Cow;
/// use cesu8::from_cesu8;
///
/// let str = "Hello, world!";
/// // Since 'str' is valid UTF-8 and CESU-8 data, 'from_cesu8' can decode
/// // the string slice without allocating memory.
/// assert_eq!(from_cesu8(str.as_bytes()), Ok(Cow::Borrowed(str)));
///
/// let str = "\u{10400}";
/// let cesu8_data = &[0xED, 0xA0, 0x81, 0xED, 0xB0, 0x80];
/// // 'cesu8_data' is a byte slice containing a 6-byte surrogate pair which
/// // becomes a 4-byte UTF-8 character.
/// assert_eq!(from_cesu8(cesu8_data), Ok(Cow::Borrowed(str)));
/// ```
#[inline]
pub fn from_cesu8(bytes: &[u8]) -> Result<Cow<str>, DecodingError> {
    from_utf8(bytes)
        .map(Cow::Borrowed)
        .or_else(|_| decode_cesu8(bytes).map(Cow::Owned))
}

#[inline(never)]
#[cold]
#[allow(clippy::unnested_or_patterns)]
fn decode_cesu8(bytes: &[u8]) -> Result<String, DecodingError> {
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut iter = bytes.iter();

    macro_rules! err {
        () => {{
            return Err(DecodingError);
        }};
    }

    macro_rules! next {
        () => {
            match iter.next() {
                Some(&byte) => byte,
                None => err!(),
            }
        };
    }

    macro_rules! next_continuation {
        () => {{
            let byte = next!();
            if is_continuation_byte(byte) {
                byte
            } else {
                err!();
            }
        }};
    }

    while let Some(&first) = iter.next() {
        if first <= MAX_ASCII_CODE_POINT {
            decoded.push(first);
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
    Ok(unsafe { String::from_utf8_unchecked(decoded) })
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
/// than [`(&str).as_bytes()`](str::as_bytes); this means that `to_cesu8` does
/// not need to perform any further operations and doesn't need to allocate any
/// additional memory.
///
/// If the string slice's representation in UTF-8 is not equivalent in CESU-8,
/// `to_cesu8` encodes the string slice to its CESU-8 representation as a slice
/// of bytes.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use std::borrow::Cow;
/// use cesu8::to_cesu8;
///
/// let str = "Hello, world!";
/// // Since 'str' is valid UTF-8 and CESU-8 data, 'to_cesu8' can encode
/// // data without allocating memory.
/// assert_eq!(to_cesu8(str), Cow::Borrowed(str.as_bytes()));
///
/// let utf8_data = "\u{10401}";
/// let cesu8_data = Cow::Borrowed(&[0xED, 0xA0, 0x81, 0xED, 0xB0, 0x81]);
/// // 'utf8_data' is a 4-byte UTF-8 representation, which becomes a 6-byte
/// // CESU-8 representation.
/// assert_eq!(to_cesu8(utf8_data), cesu8_data);
/// ```
#[must_use]
#[inline]
pub fn to_cesu8(str: &str) -> Cow<[u8]> {
    if is_valid_cesu8(str) {
        Cow::Borrowed(str.as_bytes())
    } else {
        Cow::Owned(encode_cesu8(str))
    }
}

#[must_use]
#[inline(never)]
#[cold]
fn encode_cesu8(str: &str) -> Vec<u8> {
    let bytes = str.as_bytes();
    let capacity = cesu8_len(str);
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
                encoded.extend(&bytes[slice_range]);
            } else {
                let str = &str[slice_range];
                let code_point = str.chars().next().unwrap() as u32;
                let surrogate_pair = to_surrogate_pair(code_point);
                let encoded_pair = encode_surrogate_pair(surrogate_pair);
                encoded.extend(&encoded_pair);
            }
            index += width;
        }
    }

    encoded
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

/// Returns how many bytes in CESU-8 are required to encode a string slice.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use cesu8::cesu8_len;
///
/// // Any codepoint below or equal to U+FFFF is the same length as it is in
/// // UTF-8.
/// assert_eq!(3, cesu8_len("\u{FFFF}"));
///
/// // Any codepoint above U+FFFF is stored as a surrogate pair.
/// assert_eq!(6, cesu8_len("\u{10000}"));
/// ```
#[must_use]
pub fn cesu8_len(str: &str) -> usize {
    let bytes = str.as_bytes();
    let mut len = 0;
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte <= MAX_ASCII_CODE_POINT {
            len += 1;
            index += 1;
        } else {
            // SAFETY: Valid UTF-8 will never yield a `None` value:
            let width = unsafe { utf8_char_width(byte).unwrap_unchecked() };
            len += if width <= CESU8_MAX_CHAR_WIDTH {
                width
            } else {
                6
            };
            index += width;
        }
    }
    len
}

/// Returns `true` if a string slice contains UTF-8 data that is also valid
/// CESU-8.
///
/// This is primarily used in testing if a string slice needs to be
/// explicitly encoded using [`to_cesu8`](to_cesu8). If `is_valid_cesu8()`
/// returns `false`, it implies that [`&str.as_bytes()`](str::as_bytes) is
/// directly equivalent to the string slice's CESU-8 representation.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use cesu8::is_valid_cesu8;
///
/// // Any code point below or equal to U+FFFF encoded in UTF-8 IS valid CESU-8.
/// assert!(is_valid_cesu8("Hello, world!"));
/// assert!(is_valid_cesu8("\u{FFFF}"));
///
/// // Any code point above U+FFFF encoded in UTF-8 IS NOT valid CESU-8.
/// assert!(!is_valid_cesu8("\u{10000}"));
/// ```
#[must_use]
pub fn is_valid_cesu8(str: &str) -> bool {
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

const CESU8_MAX_CHAR_WIDTH: usize = 3;

#[inline]
fn is_continuation_byte(byte: u8) -> bool {
    const TAG_MASK: u8 = 0b1100_0000;
    byte & TAG_MASK == CONT_TAG
}

const CONT_TAG: u8 = 0b1000_0000;

fn utf8_char_width(byte: u8) -> Option<usize> {
    match byte {
        0x00..=MAX_ASCII_CODE_POINT => Some(1),
        0xC2..=0xDF => Some(2),
        0xE0..=0xEF => Some(3),
        0xF0..=0xF4 => Some(4),
        _ => None,
    }
}

const MAX_ASCII_CODE_POINT: u8 = 0x7F;

/// An error thrown by [`from_cesu8`] when the input is invalid CESU-8 data.
///
/// This type does not support transmission of an error other than that an error
/// occurred.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecodingError;

impl fmt::Display for DecodingError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid CESU-8 data")
    }
}

impl Error for DecodingError {}
