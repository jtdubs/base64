#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate base64;

use std::io::{BufReader};
use arbitrary::Arbitrary;
use base64::*;

#[derive(Arbitrary)]
pub struct Parameters<'a> {
    ignore_garbage: bool,
    wrap: Option<usize>,
    data: &'a [u8]
}

fuzz_target!(|data: &[u8]| {
    let mut data_reader = BufReader::new(data);

    let mut encoded_buffer = Vec::new();
    let mut decoded_buffer = Vec::new();

    assert!(b64_encode(&mut data_reader, &mut encoded_buffer, None).is_ok());

    let mut encoded_reader = BufReader::new(encoded_buffer.as_slice());
    assert!(b64_decode(&mut encoded_reader, &mut decoded_buffer, true).is_ok());

    assert!(data == decoded_buffer, "roundtrip failed!");
});
