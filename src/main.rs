extern crate clap;

use std::io::{stdin, stdout};
use std::fs::{File};
use clap::{App, Arg};

use base64::*;

fn main() -> Result<(), std::io::Error> {
    // parse command line arguments
    let matches =
        App::new("base64")
            .version("0.0.1")
            .author("Justin Dubs <jtdubs@gmail.com>")
            .about("Base64 encode/decode data and print to standard output")
            .arg(Arg::with_name("decode")
                .short("d")
                .long("decode")
                .help("Decode data"))
            .arg(Arg::with_name("ignore_garbage")
                .short("i")
                .requires("decode")
                .long("ignore-garbage")
                .help("When decoding, ignore non-alphabet characters"))
            .arg(Arg::with_name("wrap")
                .short("w")
                .long("wrap")
                .help("Wrap encoded lines after COLS character (default 76).  Use 0 to disable line wrapping")
                .takes_value(true)
                .default_value("76")
                .validator(|arg| { arg.parse::<usize>().and(Ok(())).or(Err("wrap value must be a number".to_string())) }))
            .arg(Arg::with_name("FILE"))
            .get_matches();

    // pull out arguments
    let decode         = matches.is_present("decode");
    let ignore_garbage = matches.is_present("ignore_garbage");
    let wrap_column    = matches.value_of("wrap").unwrap().parse::<usize>().ok().filter(|&x| x != 0);
    let file           = matches.value_of("FILE").unwrap_or("-");

    // writer is always stdout
    let stdout = stdout();
    let mut writer = stdout.lock();

    // reader can be stdin or a file
    if file == "-" {
        // make a buffered reader around stdin
        let stdin = stdin();
        let mut reader = stdin.lock();

        // encode or decode as requested
        if decode {
            b64_decode(&mut reader, &mut writer, ignore_garbage)
        } else {
            b64_encode(&mut reader, &mut writer, wrap_column)
        }
    } else {
        // make a buffered reader around the file
        let mut reader = File::open(file)?;

        // encode or decode as requested
        if decode {
            b64_decode(&mut reader, &mut writer, ignore_garbage)
        } else {
            b64_encode(&mut reader, &mut writer, wrap_column)
        }
    }
}
