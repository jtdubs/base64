use std::io::{Read, Write, ErrorKind};

use crate::common::wrapping_write;

// the canonical base-64 alphabet
const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn b64_decode(reader: &mut impl Read, writer: &mut impl Write, ignore_garbage: bool) -> Result<(), std::io::Error> {
    // create reverse lookup that maps:
    //   - base-n chars back to their value (0-63)
    //   - base-n padding to 254
    //   - whitespace to 253
    let mut reverse_alphabet: [u8; 256] = [255; 256];
    for (ix, &c) in ALPHABET.into_iter().enumerate() {
        reverse_alphabet[c as usize] = ix as u8;
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
            let decoded_value: u8 = reverse_alphabet[b as usize];

            match decoded_value {
                // invalid base-n character
                255 => {
                    // either skip or error out depending on ignore_garbage flag
                    if ignore_garbage {
                        continue;
                    } else {
                        return Err(std::io::Error::new(ErrorKind::Other, "invalid input"));
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
                // base-n characters
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

pub fn b64_encode(reader: &mut impl Read, writer: &mut impl Write, wrap: Option<usize>) -> Result<(), std::io::Error> {
    // sanity-check parameters
    if wrap == Some(0) {
        return Err(std::io::Error::new(ErrorKind::Other, "cannot wrap on column 0"));
    }

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
        read_buffer.copy_within((read_index-(read_index%3))..read_index, 0);

        // update read index to end of residual data
        read_index %= 3;

        // output base-n characters
        current_col = wrapping_write(&write_buffer, write_index, wrap, current_col, writer)?;
        write_index = 0;
    }

    // process remaining data
    match read_index % 3 {
        0 => { }
        1 => {
            // output last byte as two data chars and two padding chars
            let a: u8 = read_buffer[0];
            write_buffer[write_index]   = ALPHABET[(a >> 2)         as usize];
            write_buffer[write_index+1] = ALPHABET[((a & 0x3) << 4) as usize];
            write_buffer[write_index+2] = '='  as u8;
            write_buffer[write_index+3] = '='  as u8;
            write_index += 4;
        }
        2 => {
            // output last two byte as three data chars and one padding char
            let (a, b) = (read_buffer[0], read_buffer[1]);
            write_buffer[write_index]   = ALPHABET[(a >> 2)                      as usize];
            write_buffer[write_index+1] = ALPHABET[(((a & 0x3) << 4) | (b >> 4)) as usize];
            write_buffer[write_index+2] = ALPHABET[((b & 0xF) << 2)              as usize];
            write_buffer[write_index+3] = '='  as u8;
            write_index += 4;
        }
        _ => { unreachable!("impossible mod 3 value"); }
    }

    // output base-n characters
    let _ = wrapping_write(&write_buffer, write_index, wrap, current_col, writer)?;

    // add a final newline, if wrapping is enabled
    if wrap.is_some() {
        writer.write_all(b"\n")?;
    }

    Ok(())
}
