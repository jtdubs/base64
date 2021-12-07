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
        b64_encode(stdin())
    } else {
        b64_encode(File::open(file).unwrap())
    }
}

fn b64_encode(mut reader: impl Read) -> Result<(), std::io::Error> {
    let stdout = stdout();
    let mut stdout_handle = stdout.lock();

    let alphabet : Vec<u8> = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/".bytes().collect();

    let mut read_index : usize = 0;
    let mut buffer : [u8; 4096] = [0; 4096];
    let mut bytes_read : usize;

    let mut out_buffer : Vec<u8> = Vec::with_capacity(8192);

    loop {
        // fill buffer
        bytes_read = reader.read(&mut buffer[read_index..])?;

        // if out of data, exit loop
        if bytes_read == 0 {
            break;
        }

        // update read_index base on bytes_read
        read_index += bytes_read;

        // process all chunks of 3 bytes into 4 base64 characters
        for chunk in buffer[0..read_index].chunks_exact(3) {
            let (a, b, c) = (chunk[0], chunk[1], chunk[2]);
            let abc : u32 = ((a as u32) << 16) | ((b as u32) << 8) | (c as u32);
            out_buffer.push(alphabet[ (abc >> 18)         as usize]);
            out_buffer.push(alphabet[((abc >> 12) & 0x3f) as usize]);
            out_buffer.push(alphabet[((abc >>  6) & 0x3f) as usize]);
            out_buffer.push(alphabet[( abc        & 0x3f) as usize]);
        }

        // move risidual data to front of buffer
        match read_index % 3 {
            0 => { }
            1 => { buffer[0] = buffer[read_index-1]; }
            2 => { buffer[0] = buffer[read_index-2]; buffer[1] = buffer[read_index-1]; }
            _ => { unreachable!("impossible mod 3 value"); }
        }

        // update read index to end of risidual data
        read_index %= 3;

        // write base64 characters to stdout
        stdout_handle.write_all(&out_buffer[0..out_buffer.len()])?;
        out_buffer.clear();
    }

    // process remaining data
    match read_index % 3 {
        0 => { }
        1 => {
            let (a, b, c) = (buffer[0], 0, 0);
            let abc : u32 = ((a as u32) << 16) | ((b as u32) << 8) | (c as u32);
            out_buffer.push(alphabet[ (abc >> 18)         as usize]);
            out_buffer.push(alphabet[((abc >> 12) & 0x3f) as usize]);
            out_buffer.push('=' as u8);
            out_buffer.push('=' as u8);
            out_buffer.push('\n' as u8);
        }
        2 => {
            let (a, b, c) = (buffer[0], buffer[1], 0);
            let abc : u32 = ((a as u32) << 16) | ((b as u32) << 8) | (c as u32);
            out_buffer.push(alphabet[ (abc >> 18)         as usize]);
            out_buffer.push(alphabet[((abc >> 12) & 0x3f) as usize]);
            out_buffer.push(alphabet[((abc >>  6) & 0x3f) as usize]);
            out_buffer.push('=' as u8);
            out_buffer.push('\n' as u8);
        }
        _ => { unreachable!("impossible mod 3 value"); }
    }

    stdout_handle.write_all(&out_buffer[0..out_buffer.len()])?;

    Ok(())
}
