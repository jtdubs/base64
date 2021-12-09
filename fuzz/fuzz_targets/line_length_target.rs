#![no_main]
extern crate libfuzzer_sys;
extern crate base64;

use libfuzzer_sys::fuzz_target;
use std::io::{BufRead, BufReader, Cursor};
use arbitrary::Arbitrary;
use base64::*;

#[derive(Arbitrary, Debug)]
pub struct Parameters<'a> {
    wrap: usize,
    data: &'a [u8]
}

fuzz_target!(|params: Parameters| {
    let wrap = if params.wrap == 0 { None } else { Some(params.wrap) };

    let mut data_reader = BufReader::new(params.data);

    let mut encoded_buffer = Vec::new();

    // println!("encoding: {:?}", params.data);
    assert!(b64_encode(&mut data_reader, &mut encoded_buffer, wrap).is_ok());
    // println!("encoded: {:?}", encoded_buffer.as_slice());

    let lines = Cursor::new(encoded_buffer.as_slice()).lines();

    if let Some(line_len) = wrap {
        let mut last_line = false;
        for line in lines {
            let l = line.unwrap();
            assert!(l.len() == line_len || (l.len() < line_len && !last_line), "expected: {}, got: {}", line_len, l.len());
            if l.len() < line_len {
                last_line = true;
            }
        }
    } else {
        assert!(lines.count() == 1);
    }
});
