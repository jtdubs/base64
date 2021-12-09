#![no_main]

use libfuzzer_sys::{fuzz_target};
use std::io::{BufRead, BufReader, Cursor};
use arbitrary::Arbitrary;
use base64::*;

#[derive(Arbitrary, Debug)]
pub struct Parameters<'a> {
    wrap: usize,
    data: &'a [u8]
}

fuzz_target!(|params: Parameters| {
});
