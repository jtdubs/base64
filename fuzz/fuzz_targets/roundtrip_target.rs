#![no_main]

use libfuzzer_sys::{fuzz_target};
use std::io::{BufReader};
use arbitrary::Arbitrary;
use base64::*;

#[derive(Arbitrary, Debug)]
pub struct Parameters<'a> {
    ignore_garbage: bool,
    wrap: usize,
    data: &'a [u8]
}

fuzz_target!(|params: Parameters| {
    assert!(fuzz_fn(params).is_ok());
});

fn fuzz_fn(params: Parameters) -> Result<(), std::io::Error> {
    let wrap = if params.wrap == 0 { None } else { Some(params.wrap) };

    let mut data_reader = BufReader::new(params.data);

    let mut encoded_buffer = Vec::new();
    let mut decoded_buffer = Vec::new();

    b64_encode(&mut data_reader, &mut encoded_buffer, wrap)?;

    let mut encoded_reader = BufReader::new(encoded_buffer.as_slice());
    b64_decode(&mut encoded_reader, &mut decoded_buffer, params.ignore_garbage)?;

    assert!(params.data == decoded_buffer, "roundtrip failed!");

    Ok(())
}
