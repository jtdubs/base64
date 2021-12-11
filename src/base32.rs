use std::io::{ErrorKind, Read, Write};

use crate::common::wrapping_write;

// the canonical base-64 alphabet
const ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

// reverse lookup that maps:
// - base-32 chars back to their values (0-31)
// - base-32 padding to 254
// - whitespace to 253
const REVERSE_ALPHABET: [u8; 256] = [
    //         0     1     2     3     4     5     6     7     8     9     A     B     C     D     E     F
    /* 0 */
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFD, 0xFD, 0xFD, 0xFD, 0xFD, 0xFF, 0xFF,
    /* 1 */ 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, /* 2 */ 0xFD, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, /* 3 */ 0xFF, 0xFF, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E,
    0x1F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE, 0xFF, 0xFF, /* 4 */ 0xFF, 0x00, 0x01, 0x02,
    0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, /* 5 */ 0x0F,
    0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    /* 6 */ 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, /* 7 */ 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, /* 8 */ 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, /* 9 */ 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, /* A */ 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    /* B */ 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, /* C */ 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, /* D */ 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, /* E */ 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, /* F */ 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
];

pub fn b32_decode(
    reader: &mut impl Read,
    writer: &mut impl Write,
    ignore_garbage: bool,
) -> Result<(), std::io::Error> {
    // read & write buffers
    let mut read_buffer: [u8; 65536] = [0; 65536];
    let mut write_buffer: [u8; 65536] = [0; 65536];
    let mut write_index: usize = 0; // current index into write_buffer
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
            let decoded_value: u8 = REVERSE_ALPHABET[b as usize];

            match decoded_value {
                // invalid base-64 character
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
                // base-32 characters
                _ => {
                    // store decoded_value
                    match write_offset {
                        0 => {
                            write_buffer[write_index] = decoded_value << 3;
                        }
                        1 => {
                            write_buffer[write_index] |= decoded_value << 2;
                        }
                        2 => {
                            write_buffer[write_index] |= decoded_value << 1;
                        }
                        3 => {
                            write_buffer[write_index] |= decoded_value;
                        }
                        4 => {
                            write_buffer[write_index] |= decoded_value >> 1;
                            write_buffer[write_index + 1] = decoded_value << 7;
                        }
                        5 => {
                            write_buffer[write_index] |= decoded_value >> 2;
                            write_buffer[write_index + 1] = decoded_value << 6;
                        }
                        6 => {
                            write_buffer[write_index] |= decoded_value >> 3;
                            write_buffer[write_index + 1] = decoded_value << 5;
                        }
                        7 => {
                            write_buffer[write_index] |= decoded_value >> 4;
                            write_buffer[write_index + 1] = decoded_value << 4;
                        }
                        _ => {}
                    }

                    // advance by 5 bits
                    if write_offset <= 2 {
                        write_offset += 5;
                    } else {
                        write_offset -= 3;
                        write_index += 1;
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

pub fn b32_encode(
    reader: &mut impl Read,
    writer: &mut impl Write,
    wrap: Option<usize>,
) -> Result<(), std::io::Error> {
    // sanity-check parameters
    if wrap == Some(0) {
        return Err(std::io::Error::new(
            ErrorKind::Other,
            "cannot wrap on column 0",
        ));
    }

    // read and write buffers and indecies
    let mut read_buffer: [u8; 65535] = [0; 65535];
    let mut read_index: usize = 0;
    let mut write_buffer: [u8; 104856] = [0; 104856];
    let mut write_index: usize = 0;

    // current output column (for wrapping)
    let mut current_col: usize = 0;

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
        for chunk in read_buffer[0..read_index].chunks_exact(5) {
            let (a, b, c, d, e) = (chunk[0], chunk[1], chunk[2], chunk[3], chunk[4]);
            write_buffer[write_index] = ALPHABET[(a >> 3) as usize];
            write_buffer[write_index + 1] = ALPHABET[(((a & 0x7) << 2) | (b >> 6)) as usize];
            write_buffer[write_index + 2] = ALPHABET[((b & 0x3F) >> 1) as usize];
            write_buffer[write_index + 3] = ALPHABET[(((b & 0x1) << 4) | (c >> 4)) as usize];
            write_buffer[write_index + 4] = ALPHABET[(((c & 0xF) << 1) | (d >> 7)) as usize];
            write_buffer[write_index + 5] = ALPHABET[((d & 0x7F) >> 2) as usize];
            write_buffer[write_index + 6] = ALPHABET[(((d & 0x3) << 3) | (e >> 5)) as usize];
            write_buffer[write_index + 7] = ALPHABET[(e & 0x1F) as usize];
            write_index += 8;
        }

        // move residual data to front of buffer
        read_buffer.copy_within((read_index - (read_index % 5))..read_index, 0);

        // update read index to end of residual data
        read_index %= 5;

        // output base64 characters
        current_col = wrapping_write(&write_buffer, write_index, wrap, current_col, writer)?;
        write_index = 0;
    }

    // process remaining data
    match read_index % 5 {
        0 => {}
        1 => {
            let a = read_buffer[0];
            write_buffer[write_index] = ALPHABET[(a >> 3) as usize];
            write_buffer[write_index + 1] = ALPHABET[((a & 0x7) << 2) as usize];
            write_buffer[write_index + 2] = '=' as u8;
            write_buffer[write_index + 3] = '=' as u8;
            write_buffer[write_index + 4] = '=' as u8;
            write_buffer[write_index + 5] = '=' as u8;
            write_buffer[write_index + 6] = '=' as u8;
            write_buffer[write_index + 7] = '=' as u8;
            write_index += 8;
        }
        2 => {
            let (a, b) = (read_buffer[0], read_buffer[1]);
            write_buffer[write_index] = ALPHABET[(a >> 3) as usize];
            write_buffer[write_index + 1] = ALPHABET[(((a & 0x7) << 2) | (b >> 6)) as usize];
            write_buffer[write_index + 2] = ALPHABET[((b & 0x3F) >> 1) as usize];
            write_buffer[write_index + 3] = ALPHABET[((b & 0x1) << 4) as usize];
            write_buffer[write_index + 4] = '=' as u8;
            write_buffer[write_index + 5] = '=' as u8;
            write_buffer[write_index + 6] = '=' as u8;
            write_buffer[write_index + 7] = '=' as u8;
            write_index += 8;
        }
        3 => {
            let (a, b, c) = (read_buffer[0], read_buffer[1], read_buffer[2]);
            write_buffer[write_index] = ALPHABET[(a >> 3) as usize];
            write_buffer[write_index + 1] = ALPHABET[(((a & 0x7) << 2) | (b >> 6)) as usize];
            write_buffer[write_index + 2] = ALPHABET[((b & 0x3F) >> 1) as usize];
            write_buffer[write_index + 3] = ALPHABET[(((b & 0x1) << 4) | (c >> 4)) as usize];
            write_buffer[write_index + 4] = ALPHABET[((c & 0xF) << 1) as usize];
            write_buffer[write_index + 5] = '=' as u8;
            write_buffer[write_index + 6] = '=' as u8;
            write_buffer[write_index + 7] = '=' as u8;
            write_index += 8;
        }
        4 => {
            let (a, b, c, d) = (
                read_buffer[0],
                read_buffer[1],
                read_buffer[2],
                read_buffer[3],
            );
            write_buffer[write_index] = ALPHABET[(a >> 3) as usize];
            write_buffer[write_index + 1] = ALPHABET[(((a & 0x7) << 2) | (b >> 6)) as usize];
            write_buffer[write_index + 2] = ALPHABET[((b & 0x3F) >> 1) as usize];
            write_buffer[write_index + 3] = ALPHABET[(((b & 0x1) << 4) | (c >> 4)) as usize];
            write_buffer[write_index + 4] = ALPHABET[(((c & 0xF) << 1) | (d >> 7)) as usize];
            write_buffer[write_index + 5] = ALPHABET[((d & 0x7F) >> 2) as usize];
            write_buffer[write_index + 6] = ALPHABET[((d & 0x3) << 3) as usize];
            write_buffer[write_index + 7] = '=' as u8;
            write_index += 8;
        }
        _ => {
            unreachable!("impossible mod 3 value");
        }
    }

    // output base64 characters
    let _ = wrapping_write(&write_buffer, write_index, wrap, current_col, writer)?;

    // add a final newline, if wrapping is enabled
    if wrap.is_some() {
        writer.write_all(b"\n")?;
    }

    Ok(())
}
