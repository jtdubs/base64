use clap::{App, Arg};
use std::fs::File;
use std::io::{stdin, stdout, BufWriter, Read, Stdin};

use base_util::*;

fn main() {
    if let Err(e) = app() {
        eprintln!("base32: {}", e);
        std::process::exit(1);
    }
}

fn app() -> Result<(), std::io::Error> {
    // parse command line arguments
    let matches =
        App::new("base32")
            .version("0.0.1")
            .author("Justin Dubs <jtdubs@gmail.com>")
            .about("Base32 encode/decode data and print to standard output")
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
    let decode = matches.is_present("decode");
    let ignore_garbage = matches.is_present("ignore_garbage");
    let wrap_column = matches
        .value_of("wrap")
        .unwrap()
        .parse::<usize>()
        .ok()
        .filter(|&x| x != 0);
    let file = matches.value_of("FILE").unwrap_or("-");

    // writer is always stdout
    let stdout = stdout();
    let stdout_lock = stdout.lock();
    let mut writer = BufWriter::new(stdout_lock);

    // reader is either stdin or input file
    let stdin = stdin();
    let mut reader = get_reader(file, &stdin)?;

    // encode or decode as requested
    if decode {
        b32_decode(&mut reader, &mut writer, ignore_garbage)?;
    } else {
        b32_encode(&mut reader, &mut writer, wrap_column)?;
    }

    Ok(())
}

fn get_reader<'a>(file: &str, stdin: &'a Stdin) -> Result<Box<dyn Read + 'a>, std::io::Error> {
    if file == "-" {
        Ok(Box::new(stdin.lock()))
    } else {
        let file = File::open(file)?;
        Ok(Box::new(file))
    }
}
