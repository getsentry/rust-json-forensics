use std::io::{self, Read};
use std::str;

use json_forensics::translate_slice;

fn main() {
    let mut buffer = vec![];
    io::stdin().read_to_end(&mut buffer).unwrap();
    let old_buffer = buffer.clone();

    translate_slice(&mut buffer[..]);
    assert_eq!(str::from_utf8(&buffer[..]), str::from_utf8(&old_buffer[..]));
}
