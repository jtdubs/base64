use std::io::{BufReader, BufWriter};
use base_util::*;

fn encode(input: &[u8], wrap: Option<usize>) -> Result<Vec<u8>, std::io::Error> {
    let output = Vec::new();

    let mut reader = BufReader::new(input);
    let mut writer = BufWriter::new(output);

    b64_encode(&mut reader, &mut writer, wrap)?;

    Ok(writer.buffer().to_vec())
}

fn decode(input: &[u8], ignore_garbage: bool) -> Result<Vec<u8>, std::io::Error> {
    let output = Vec::new();

    let mut reader = BufReader::new(input);
    let mut writer = BufWriter::new(output);

    b64_decode(&mut reader, &mut writer, ignore_garbage)?;

    Ok(writer.buffer().to_vec())
}

fn test_encode(input: &[u8], expected: &[u8], wrap: Option<usize>) {
    assert_eq!(encode(input, wrap).unwrap(), expected.to_vec());
}

fn test_decode(input: &[u8], expected: &[u8], ignore_garbage: bool) {
    assert_eq!(decode(input, ignore_garbage).unwrap(), expected);
}

fn test_encode_err(input: &[u8], wrap: Option<usize>) {
    assert!(encode(input, wrap).is_err());
}

fn test_decode_err(input: &[u8], ignore_garbage: bool) {
    assert!(decode(input, ignore_garbage).is_err());
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

#[test]
fn test_err_on_invalid_char() {
    test_decode_err(b"VGhlIHF1a^NrIGJyb3du\nIGZveCBqdW1wcyBvdmVy\nIHRoZSBsYXp5IGRvZy4=\n", false);
}

#[test]
fn test_ignore_invalid_char() {
    test_decode(b"VGhlIHF1aWNrIGJyb3du^^\nIGZveCBqdW1wcyBvdmVy\nIHRoZSBsYXp5IGRvZy4=\n", b"The quick brown fox jumps over the lazy dog.", true);
}

#[test]
fn test_invalid_wrap() {
    test_encode_err(b"hello world", Some(0));
}
