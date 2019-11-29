//! This crate implements a `Read` adapter that converts the invalid JSON
//! tokens `NaN` and `Infinity` into other tokens without otherwise distorting
//! the stream.  It achieves this by converting `NaN` and `Infinity` into `0.0`.
//!
//! This is useful because the Python JSON library traditionally emits invalid
//! JSON if `NaN` and `Infinity` values are encountered.  If you have to support
//! clients like this, this wrapper can be used to still deserialize such a
//! JSON document.
//!
//! This is just a way to get this to parse and `0` is a value that can
//! be inserted in a standardized way that fits without changing any of the
//! positions.
//!
//! # Example Conversion
//!
//! The following JSON document:
//!
//! ```ignore
//! {"nan":NaN,"inf":Infinity,"-inf":-Infinity}
//! ```
//!
//! is thus converted to:
//!
//! ```ignore
//! {"nan":0  ,"inf":0       ,"-inf":-0       }
//! ```
//!
//! # serde support
//!
//! If the `serde` feature is enabled then the crate provides some basic
//! wrappers around `serde_json` to deserialize quickly and also by running
//! the conversions.

use std::str;

#[cfg(feature = "serde")]
mod serde_impl;
#[cfg(feature = "serde")]
pub use self::serde_impl::*;

#[derive(Copy, Clone)]
enum State {
    Initial,
    Quoted,
    QuotedEscape,
    NaN0,
    NaN1,
    Number { start: usize },
    Infinity0,
    Infinity1,
    Infinity2,
    Infinity3,
    Infinity4,
    Infinity5,
    Infinity6,
}

#[inline]
fn transition(bytes: &mut [u8], state: State, i: usize, c: u8) -> (State, u8) {
    match (state, c) {
        (State::Initial, b'N') => (State::NaN0, b'N'),
        (State::NaN0, b'a') => (State::NaN1, b'a'),
        (State::NaN1, b'N') => {
            bytes[i - 2] = b'0';
            bytes[i - 1] = b' ';
            (State::Initial, b' ')
        }
        (State::Initial, b'I') => (State::Infinity0, b'I'),
        (State::Infinity0, b'n') => (State::Infinity1, b'n'),
        (State::Infinity1, b'f') => (State::Infinity2, b'f'),
        (State::Infinity2, b'i') => (State::Infinity3, b'i'),
        (State::Infinity3, b'n') => (State::Infinity4, b'n'),
        (State::Infinity4, b'i') => (State::Infinity5, b'i'),
        (State::Infinity5, b't') => (State::Infinity6, b't'),
        (State::Infinity6, b'y') => {
            bytes[i - 7] = b'0';
            for j in (i - 6)..i {
                bytes[j] = b' ';
            }
            (State::Initial, b' ')
        }
        (State::Initial, b'"') => (State::Quoted, b'"'),
        (State::Quoted, b'\\') => (State::QuotedEscape, b'\\'),
        (State::QuotedEscape, c) => (State::Quoted, c),
        (State::Quoted, b'"') => (State::Initial, b'"'),
        (State::Initial, c) if c.is_ascii_digit() => (State::Number { start: i }, c),
        (State::Number { .. }, b'.') => (State::Initial, b'.'),
        (State::Number { .. }, b'E') => (State::Initial, b'E'),
        (State::Number { .. }, b'e') => (State::Initial, b'e'),
        (State::Number { start }, c) if !c.is_ascii_digit() => {
            if let Ok(num_str) = str::from_utf8(&bytes[start..i]) {
                if num_str.parse::<u64>().is_err() && num_str.parse::<i64>().is_err() {
                    bytes[start] = b'0';
                    for j in (start + 1)..i {
                        bytes[j] = b' ';
                    }
                }
            }

            (State::Initial, c)
        }
        (state, c) => (state, c),
    }
}

fn translate_slice_impl(bytes: &mut [u8], mut state: State) -> State {
    for i in 0..bytes.len() {
        let (new_state, new_char) = transition(bytes, state, i, bytes[i]);
        state = new_state;
        bytes[i] = new_char;
    }
    transition(bytes, state, bytes.len(), b'\0');
    state
}

/// Translates a slice in place.
///
/// This works the same as the `JsonCompatRead` struct but instead converts a
/// slice in place.  This is useful when working with JSON in slices.
pub fn translate_slice(bytes: &mut [u8]) {
    translate_slice_impl(bytes, State::Initial);
}

#[test]
fn test_reader_simple() {
    let mut json = br#"{"nan":0.0,"inf":Infinity,"-inf":-Infinity}"#.to_vec();
    translate_slice(&mut json[..]);
    assert_eq!(
        str::from_utf8(&json[..]),
        str::from_utf8(&b"{\"nan\":0.0,\"inf\":0       ,\"-inf\":-0       }"[..])
    );
}

#[test]
fn test_reader_string() {
    let mut json = br#"{"nan":"nan","Infinity":"-Infinity","other":NaN}"#.to_vec();
    translate_slice(&mut json[..]);
    assert_eq!(
        &json[..],
        &b"{\"nan\":\"nan\",\"Infinity\":\"-Infinity\",\"other\":0  }"[..]
    );
}

#[test]
fn test_reader_string_escaping() {
    let mut json = br#""NaN\"NaN\"NaN""#.to_vec();
    translate_slice(&mut json[..]);
    assert_eq!(&json[..], &br#""NaN\"NaN\"NaN""#[..]);
}

#[test]
fn test_no_greedy_write() {
    let mut json = br#"Inferior"#.to_vec();
    translate_slice(&mut json[..]);
    assert_eq!(&json[..], &b"Inferior"[..]);
}

#[test]
fn test_too_large_int() {
    let mut json = br#"999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999"#.to_vec();
    translate_slice(&mut json[..]);
    assert_eq!(str::from_utf8(&json[..]), str::from_utf8(
                    &b"0                                                                                                              "[..]));
}

#[test]
fn test_leaves_floats() {
    let mut json = br#"9999999999999999999999999999.99999"#.to_vec();
    let old_json = json.clone();
    translate_slice(&mut json[..]);
    assert_eq!(str::from_utf8(&json[..]), str::from_utf8(&old_json[..]));
}

#[test]
fn test_leaves_floats2() {
    let mut json = br#"999999999E10"#.to_vec();
    let old_json = json.clone();
    translate_slice(&mut json[..]);
    assert_eq!(str::from_utf8(&json[..]), str::from_utf8(&old_json[..]));
}

#[test]
fn test_leaves_floats3() {
    let mut json = br#"999999999E-10"#.to_vec();
    let old_json = json.clone();
    translate_slice(&mut json[..]);
    assert_eq!(str::from_utf8(&json[..]), str::from_utf8(&old_json[..]));
}

#[test]
fn test_leaves_floats4() {
    let mut json = br#"999999999e-10"#.to_vec();
    let old_json = json.clone();
    translate_slice(&mut json[..]);
    assert_eq!(str::from_utf8(&json[..]), str::from_utf8(&old_json[..]));
}
