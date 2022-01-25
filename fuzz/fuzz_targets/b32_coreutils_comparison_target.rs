#![no_main]

use arbitrary::Arbitrary;
use base_util::*;
use libfuzzer_sys::fuzz_target;
use std::io::{BufReader, Write};
use std::process::Command;
use tempfile::NamedTempFile;

#[derive(Arbitrary, Debug)]
pub struct Parameters<'a> {
    wrap: usize,
    data: &'a [u8],
}

fuzz_target!(|params: Parameters| {
    assert!(fuzz_fn(params).is_ok());
});

fn fuzz_fn(params: Parameters) -> Result<(), std::io::Error> {
    // put data in temp file
    let mut file = NamedTempFile::new()?;
    file.write_all(params.data)?;

    // encode it ourselves
    let wrap = if params.wrap == 0 {
        None
    } else {
        Some(params.wrap)
    };
    let mut data_reader = BufReader::new(params.data);
    let mut encoded_buffer = Vec::new();
    b32_encode(&mut data_reader, &mut encoded_buffer, wrap)?;

    // encode it with coreutils base32
    let result = Command::new("base32")
        .arg("-w")
        .arg(params.wrap.to_string())
        .stdin(file.reopen()?)
        .output()?;

    // compare results
    assert!(
        encoded_buffer.as_slice() == result.stdout.as_slice(),
        "expected: {:?}, got: {:?}",
        result.stdout.as_slice(),
        encoded_buffer.as_slice()
    );

    Ok(())
}
