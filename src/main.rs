extern crate clap;
use clap::{App, Arg};
use std::io::{Read, Write, stdin, stdout};
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
        let stdin = stdin();
        let stdin_handle = stdin.lock();
        b64_encode(stdin_handle)
    } else {
        b64_encode(File::open(file).unwrap())
    }
}

fn b64_encode(mut reader: impl Read) -> Result<(), std::io::Error> {
    let alphabet : Vec<u8> = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/".bytes().collect();

    let stdout = stdout();
    let mut stdout_handle = stdout.lock();

    let mut read_buffer  : [u8; 65536]  = [0; 65536];
    let mut read_index   : usize        = 0;
    let mut write_buffer : [u8; 131072] = [0; 131072];
    let mut write_index  : usize        = 0;

    loop {
        // fill buffer
        let bytes_read = reader.read(&mut read_buffer[read_index..])?;

        // if out of data, exit loop
        if bytes_read == 0 {
            break;
        }

        // update read_index base on bytes_read
        read_index += bytes_read;

        // process all chunks of 3 bytes into 4 base64 characters
        for chunk in read_buffer[0..read_index].chunks_exact(3) {
            let (a, b, c) = (chunk[0], chunk[1], chunk[2]);
            write_buffer[write_index]   = alphabet[(a >> 2)                      as usize];
            write_buffer[write_index+1] = alphabet[(((a & 0x3) << 4) | (b >> 4)) as usize];
            write_buffer[write_index+2] = alphabet[(((b & 0xF) << 2) | (c >> 6)) as usize];
            write_buffer[write_index+3] = alphabet[(c & 0x3F)                    as usize];
            write_index += 4;
        }

        // move risidual data to front of buffer
        match read_index % 3 {
            0 => { }
            1 => { read_buffer[0] = read_buffer[read_index-1]; }
            2 => { read_buffer[0] = read_buffer[read_index-2]; read_buffer[1] = read_buffer[read_index-1]; }
            _ => { unreachable!("impossible mod 3 value"); }
        }

        // update read index to end of risidual data
        read_index %= 3;

        // write base64 characters to stdout
        stdout_handle.write_all(&write_buffer[0..write_index])?;
        write_index = 0;
    }

    // process remaining data
    match read_index % 3 {
        0 => { }
        1 => {
            let a = read_buffer[0];
            write_buffer[write_index]   = alphabet[(a >> 2)         as usize];
            write_buffer[write_index+1] = alphabet[((a & 0x3) << 4) as usize];
            write_buffer[write_index+2] = '=' as u8;
            write_buffer[write_index+3] = '=' as u8;
            write_buffer[write_index+4] = '\n' as u8;
            write_index += 5;
        }
        2 => {
            let (a, b) = (read_buffer[0], read_buffer[1]);
            write_buffer[write_index]   = alphabet[(a >> 2)                      as usize];
            write_buffer[write_index+1] = alphabet[(((a & 0x3) << 4) | (b >> 4)) as usize];
            write_buffer[write_index+2] = alphabet[((b & 0xF) << 2)              as usize];
            write_buffer[write_index+3] = '=' as u8;
            write_buffer[write_index+4] = '\n' as u8;
            write_index += 5;
        }
        _ => { unreachable!("impossible mod 3 value"); }
    }

    stdout_handle.write_all(&write_buffer[0..write_index])?;

    Ok(())
}
