extern crate clap;
use clap::{App, Arg};
use std::io::{Read, stdin};
use std::fs::{File};

fn main() -> Result<(), std::io::Error> {
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
                .validator(|arg| { arg.parse::<u32>().and(Ok(())).or(Err("wrap value must be a number".to_string())) }))
            .arg(Arg::with_name("FILE"))
            .get_matches();

    let _decode         = matches.is_present("decode");
    let _ignore_garbage = matches.is_present("ignore_garbage");
    let _wrap_column    = matches.value_of("wrap").unwrap().parse::<u32>().unwrap_or(76);
    let file           = matches.value_of("FILE").unwrap_or("-");

    if file == "-" {
        b64_encode(stdin())
    } else {
        b64_encode(File::open(file).unwrap())
    }
}

fn b64_encode(mut reader: impl Read) -> Result<(), std::io::Error> {
    let alphabet : Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/".chars().collect();

    let mut read_index : usize = 0;
    let mut buffer : [u8; 16384] = [0; 16384];
    let mut buffer_end : usize;
    let mut bytes_read : usize;

    // let mut reader = BufReader::new(input);

    loop {
        // fill buffer
        bytes_read = reader.read(&mut buffer[read_index..])?;

        // determine end index based on start and amount read
        buffer_end = bytes_read + read_index;

        // if at least one full "block" of 3 bytes
        if buffer_end >= 3 {
            // process each block
            for i in (0..buffer_end-2).step_by(3) {
                let a = buffer[i];
                let b = buffer[i+1];
                let c = buffer[i+2];
                let r1 = alphabet[(a >> 2) as usize];
                let r2 = alphabet[(((a & 0x3) << 4) | (b >> 4)) as usize];
                let r3 = alphabet[(((b & 0xF) << 2) | (c >> 6)) as usize];
                let r4 = alphabet[(c & 0x3F) as usize];
                print!("{}{}{}{}", r1, r2, r3, r4);
            }
        }

        // update read index based on risidual data
        read_index = buffer_end % 3;

        // move risidual data to front of buffer
        match buffer_end % 3 {
            0 => { }
            1 => { buffer[0] = buffer[buffer_end-1]; }
            2 => { buffer[0] = buffer[buffer_end-2]; buffer[1] = buffer[buffer_end-1]; }
            _ => { unreachable!("impossible mod 3 value"); }
        }

        // if out of data, exit loop
        if bytes_read == 0 {
            break;
        }
    }

    // process remaining data
    match buffer_end % 3 {
        0 => { }
        1 => {
            let a = buffer[0];
            let r1 = alphabet[(a >> 2) as usize];
            let r2 = alphabet[((a & 0x3) << 4) as usize];
            print!("{}{}==", r1, r2);
        }
        2 => {
            let a = buffer[0];
            let b = buffer[1];
            let r1 = alphabet[(a >> 2) as usize];
            let r2 = alphabet[(((a & 0x3) << 4) | (b >> 4)) as usize];
            let r3 = alphabet[((b & 0xF) << 2) as usize];
            print!("{}{}{}=", r1, r2, r3);
        }
        _ => { unreachable!("impossible mod 3 value"); }
    }

    println!();

    Ok(())
}
