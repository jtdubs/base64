extern crate clap;

use std::io::{Read, Write, BufReader, BufWriter, stdin, stdout, ErrorKind};
use std::fs::{File};
use clap::{App, Arg};

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
    let stdout_handle = stdout.lock();
    let writer = BufWriter::new(stdout_handle);

    // reader can be stdin or a file
    if file == "-" {
        // make a buffered reader around stdin
        let stdin = stdin();
        let stdin_lock = stdin.lock();
        let reader = BufReader::new(stdin_lock);

        // encode or decode as requested
        if decode {
            b64_decode(reader, writer, ignore_garbage)
        } else {
            b64_encode(reader, writer, wrap_column)
        }
    } else {
        // make a buffered reader around the file
        let file_handle = File::open(file)?;
        let reader = BufReader::new(file_handle);

        // encode or decode as requested
        if decode {
            b64_decode(reader, writer, ignore_garbage)
        } else {
            b64_encode(reader, writer, wrap_column)
        }
    }
}

// the canonical base-64 alphabet
const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn b64_decode(mut reader: impl Read, mut writer: impl Write, ignore_garbage: bool) -> Result<(), std::io::Error> {
    // create reverse lookup that maps:
    //   - base-64 chars back to their value (0-63)
    //   - base-64 padding to 254
    //   - whitespace to 253
    let mut reverse_alphabet: [u8; 256] = [255; 256];
    for ix in 0..64 {
        reverse_alphabet[ALPHABET[ix] as usize] = ix as u8;
    }
    reverse_alphabet['=' as usize] = 254;
    for c in " \t\r\n".bytes() {
        reverse_alphabet[c as usize] = 253;
    }

    // read & write buffers
    let mut read_buffer:  [u8; 4096] = [0; 4096];
    let mut write_buffer: [u8; 4096] = [0; 4096];
    let mut write_index:  usize = 0; // current index into write_buffer
    let mut write_offset: usize = 0; // current bit-offset within write_buffer[index]

    loop {
        // fill read buffer
        let bytes_read = reader.read(&mut read_buffer[..])?;

        // if out of data, exit loop
        if bytes_read == 0 {
            break;
        }

        // process bytes
        for &b in read_buffer[0..bytes_read].iter() {
            // decode the character
            let decoded_value = reverse_alphabet[b as usize];

            match decoded_value {
                // invalid base-64 character
                255 => {
                    // either skip or error out depending on ignore_garbage flag
                    if ignore_garbage {
                        continue;
                    } else {
                        return Err(std::io::Error::new(ErrorKind::InvalidData, "invalid base64 character encountered"));
                    }
                }
                // padding
                254 => {
                    // that means no more data is coming, so exit loop
                    break;
                }
                // whitespace
                253 => {
                    // skip it
                    continue;
                }
                // base-64 characters
                _ => {
                    // update write buffer by storing decoded_value and advancing by 6 bits
                    match write_offset {
                        0 => {
                            // store value and advance by 6 bits
                            write_buffer[write_index] = decoded_value << 2;
                            write_offset = 6;
                        }
                        2 => {
                            write_buffer[write_index] |= decoded_value;
                            write_index += 1;
                            write_offset = 0;
                        }
                        4 => {
                            write_buffer[write_index] |= decoded_value >> 2;
                            write_buffer[write_index+1] = decoded_value << 6;
                            write_index += 1;
                            write_offset = 2;
                        }
                        6 => {
                            write_buffer[write_index] |= decoded_value >> 4;
                            write_buffer[write_index+1] = decoded_value << 4;
                            write_index += 1;
                            write_offset = 4;
                        }
                        _ => { }
                    }
                }
            }
        }

        // output decoded bytes
        writer.write_all(&write_buffer[0..write_index])?;

        // reset write buffer, saving partial byte if necessary
        if write_offset != 0 {
            write_buffer[0] = write_buffer[write_index];
        }
        write_index = 0;
    }

    Ok(())
}

fn b64_encode(mut reader: impl Read, mut writer: impl Write, wrap: Option<usize>) -> Result<(), std::io::Error> {
    // read and write buffers and indecies
    let mut read_buffer:  [u8; 65535] = [0; 65535];
    let mut read_index:   usize       = 0;
    let mut write_buffer: [u8; 87380] = [0; 87380];
    let mut write_index:  usize       = 0;

    // current output column (for wrapping)
    let mut current_col:  usize = 0;

    loop {
        // fill read buffer
        let bytes_read = reader.read(&mut read_buffer[read_index..])?;

        // if out of data, exit loop
        if bytes_read == 0 {
            break;
        }

        // update read_index base on bytes_read
        read_index += bytes_read;

        // process all chunks of 3 bytes into 4 output characters
        for chunk in read_buffer[0..read_index].chunks_exact(3) {
            let (a, b, c) = (chunk[0], chunk[1], chunk[2]);
            write_buffer[write_index]   = ALPHABET[(a >> 2)                      as usize];
            write_buffer[write_index+1] = ALPHABET[(((a & 0x3) << 4) | (b >> 4)) as usize];
            write_buffer[write_index+2] = ALPHABET[(((b & 0xF) << 2) | (c >> 6)) as usize];
            write_buffer[write_index+3] = ALPHABET[(c & 0x3F)                    as usize];
            write_index += 4;
        }

        // move residual data to front of buffer
        match read_index % 3 {
            0 => { }
            1 => { read_buffer[0] = read_buffer[read_index-1]; }
            2 => { read_buffer[0] = read_buffer[read_index-2]; read_buffer[1] = read_buffer[read_index-1]; }
            _ => { unreachable!("impossible mod 3 value"); }
        }

        // update read index to end of residual data
        read_index %= 3;

        // output base64 characters
        current_col = wrapping_write(&write_buffer, write_index, wrap, current_col, &mut writer)?;
        write_index = 0;
    }

    // process remaining data
    match read_index % 3 {
        0 => { }
        1 => {
            // output last byte as two data chars and two padding chars
            let a = read_buffer[0];
            write_buffer[write_index]   = ALPHABET[(a >> 2)         as usize];
            write_buffer[write_index+1] = ALPHABET[((a & 0x3) << 4) as usize];
            write_buffer[write_index+2] = '='  as u8;
            write_buffer[write_index+3] = '='  as u8;
            write_buffer[write_index+4] = '\n' as u8;
            write_index += 5;
        }
        2 => {
            // output last two byte as three data chars and one padding char
            let (a, b) = (read_buffer[0], read_buffer[1]);
            write_buffer[write_index]   = ALPHABET[(a >> 2)                      as usize];
            write_buffer[write_index+1] = ALPHABET[(((a & 0x3) << 4) | (b >> 4)) as usize];
            write_buffer[write_index+2] = ALPHABET[((b & 0xF) << 2)              as usize];
            write_buffer[write_index+3] = '='  as u8;
            write_buffer[write_index+4] = '\n' as u8;
            write_index += 5;
        }
        _ => { unreachable!("impossible mod 3 value"); }
    }

    // output base64 characters
    let _ = wrapping_write(&write_buffer, write_index, wrap, current_col, &mut writer)?;

    Ok(())
}

fn wrapping_write(buffer: &[u8], len: usize, wrap_col: Option<usize>, mut current_col: usize, writer: &mut impl Write) -> Result<usize, std::io::Error> {
    // if wrapping is required
    if let Some(line_length) = wrap_col {
        let mut written = 0;

        // while there are more bytes to write
        while written < len {
            // calculate bytes remaining in line and total bytes remaining
            let line_remaining = line_length - current_col;
            let byte_remaining = len - written;

            // bytes to write this iteration is the min of those values
            let n = line_remaining.min(byte_remaining);

            // write the output
            writer.write_all(&buffer[written..written+n])?;
            written += n;

            // if a line was completed
            if n == line_remaining {
                // add a newline and reset the column counter
                writer.write_all(b"\n")?;
                current_col = 0 as usize;
            // otherwise
            } else {
                // advance to the new column
                current_col += n;
            }
        }
    // otherwise, if no wrapping
    } else {
        // just output all the data
        writer.write_all(&buffer[0..len])?;
    }

    // return the new column
    Ok(current_col)
}
