use std::io::{BufReader, BufWriter};
use base64::*;

fn test_encode(input: &[u8], expected: &[u8], wrap: Option<usize>) {
    let output = Vec::new();

    let mut reader = BufReader::new(input);
    let mut writer = BufWriter::new(output);

    assert!(b64_encode(&mut reader, &mut writer, wrap).is_ok());
    assert_eq!(writer.buffer(), expected);
}

fn test_decode(input: &[u8], expected: &[u8], ignore_garbage: bool) {
    let output = Vec::new();

    let mut reader = BufReader::new(input);
    let mut writer = BufWriter::new(output);

    assert!(b64_decode(&mut reader, &mut writer, ignore_garbage).is_ok());
    assert_eq!(writer.buffer(), expected);
}

fn test_bidi(data: &[u8], encoded: &[u8], wrap: Option<usize>, ignore_garbage: bool) {
    test_encode(data, encoded, wrap);
    test_decode(encoded, data, ignore_garbage);
}

fn test_bidi_simple(data: &[u8], encoded: &[u8]) {
    test_bidi(data, encoded, None, false);
}


#[test]
fn test_empty() {
    test_bidi_simple(&[], &[]);
}

#[test]
fn test_hello_world() {
    test_bidi_simple(b"hello, world!", b"aGVsbG8sIHdvcmxkIQ==");
}

#[test]
fn test_wrapping() {
    test_bidi(b"The quick brown fox jumps over the lazy dog.", b"VGhlIHF1aWNrIGJyb3du\nIGZveCBqdW1wcyBvdmVy\nIHRoZSBsYXp5IGRvZy4=\n", Some(20), false);
}
