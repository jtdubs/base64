use std::io::{BufReader, BufRead, Read, Write, ErrorKind};

use crate::common::wrapping_write;

// the canonical base-64 alphabet
const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

// reverse lookup that maps:
// - base-64 chars back to their values (0-63)
// - base-64 padding to 64
// - whitespace to 253
const REVERSE_ALPHABET: [u8; 256] = [
//         0     1     2     3     4     5     6     7     8     9     A     B     C     D     E     F
/* 0 */ 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFD, 0xFD, 0xFD, 0xFD, 0xFD, 0xFF, 0xFF,
/* 1 */ 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
/* 2 */ 0xFD, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x3E, 0xFF, 0xFF, 0xFF, 0x3F,
/* 3 */ 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0xFF, 0xFF, 0xFF, 0x40, 0xFF, 0xFF,
/* 4 */ 0xFF, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
/* 5 */ 0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
/* 6 */ 0xFF, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28,
/* 7 */ 0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F, 0x30, 0x31, 0x32, 0x33, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
/* 8 */ 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
/* 9 */ 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
/* A */ 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
/* B */ 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
/* C */ 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
/* D */ 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
/* E */ 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
/* F */ 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF
];

pub fn b64_decode(internal_reader: &mut impl Read, writer: &mut impl Write, ignore_garbage: bool) -> Result<(), std::io::Error> {
    let mut reader = BufReader::with_capacity(65536, internal_reader);

    // read & write buffers
    let mut write_buffer: [u8; 3] = [0; 3];

    // temp buffer
    let mut word: [u8; 4] = [0; 4];
    let mut word_index: usize = 0;

    let mut reached_end = false;

    loop {
        let buffer = reader.fill_buf()?;

        let n = buffer.len();
        if n == 0 {
            break;
        }

        for &b in buffer {
            // decode the character
            let decoded_value: u8 = REVERSE_ALPHABET[b as usize];
            word[word_index] = decoded_value;
            word_index += 1;

            // if full word possibly ready to decode
            if word_index == 4 {
                // happy path, all bytes are valid
                if (word[0] | word[1] | word[2] | word[3]) < 64 && !reached_end {
                    // decode and output word
                    write_buffer[0] = word[0] << 2 | word[1] >> 4;
                    write_buffer[1] = word[1] << 4 | word[2] >> 2;
                    write_buffer[2] = word[2] << 6 | word[3];
                    writer.write_all(&write_buffer)?;
                    word_index = 0;
                } else {
                    // clean out garbage and whitespace
                    for i in 0..4 {
                        match word[i] {
                            253 => {
                                word.copy_within((i+1)..4, i);
                                word_index -= 1;
                            }
                            255 => {
                                // either skip or error out depending on ignore_garbage flag
                                if ignore_garbage {
                                    word.copy_within((i+1)..4, i);
                                    word_index -= 1;
                                } else {
                                    return Err(std::io::Error::new(ErrorKind::Other, "invalid input"));
                                }
                            }
                            _ => { }
                        }
                    }

                    // if still full, must be padded
                    if word_index == 4 {
                        if reached_end {
                            return Err(std::io::Error::new(ErrorKind::Other, "invalid input (2)"));
                        }

                        reached_end = true;
                        word_index = 0;

                        if word[1] == 64 && word[2] == 64 && word[3] == 64 {

                        } else if word[2] == 64 && word[3] == 64 {
                            write_buffer[0] = word[0] << 2 | word[1] >> 4;
                            writer.write_all(&write_buffer[0..1])?;
                        } else if word[3] == 64 {
                            write_buffer[0] = word[0] << 2 | word[1] >> 4;
                            write_buffer[1] = word[1] << 4 | word[2] >> 2;
                            writer.write_all(&write_buffer[0..2])?;
                        } else {
                            return Err(std::io::Error::new(ErrorKind::Other, "invalid input (3)"));
                        }

                        if ignore_garbage {
                            return Ok(());
                        }
                    }
                }
            }
        }

        reader.consume(n);
    }

    if word_index != 0 && !ignore_garbage {
        for i in 0..word_index {
            if word[i] == 255 {
                return Err(std::io::Error::new(ErrorKind::Other, "invalid input"));
            }
        }
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

        // output base-64 characters
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

    // output base-64 characters
    let _ = wrapping_write(&write_buffer, write_index, wrap, current_col, writer)?;

    // add a final newline, if wrapping is enabled
    if wrap.is_some() {
        writer.write_all(b"\n")?;
    }

    Ok(())
}
