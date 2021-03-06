use std::io::{BufRead, BufReader, ErrorKind, Read, Write};

use crate::common::wrapping_write;

// the canonical base-64 alphabet
const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

// reverse lookup that maps:
// - base-64 chars back to their values (0-63)
// - base-64 padding to 64
// - whitespace to 254
// - garbage to 255
const REVERSE_ALPHABET: [u8; 256] = [
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x3E, 0xFF, 0xFF, 0xFF, 0x3F,
    0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0xFF, 0xFF, 0xFF, 0x40, 0xFF, 0xFF,
    0xFF, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
    0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28,
    0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F, 0x30, 0x31, 0x32, 0x33, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
];

///
/// Decode base-64 encoded data
///
/// # Arguments
///
/// * `reader` - Base-64 encoded data reader
/// * `writer` - Writer to which decoded data will be written
/// * `ignore_garbage` - Whether or not invalid characters should be ignored
///
pub fn b64_decode(
    reader: &mut impl Read,
    writer: &mut impl Write,
    ignore_garbage: bool,
) -> Result<(), std::io::Error> {
    // wrap the reader in a 64KiB buffered reader
    let mut buf_reader = BufReader::with_capacity(65536, reader);

    // internal write buffering to minimize calls to the writer
    let mut write_buffer: [u8; 65536] = [0; 65536];
    let mut write_index: usize = 0;

    // temp buffer for 4-char base-64 `word` to be decoded
    let mut word: [u8; 4] = [0; 4];
    let mut word_index: usize = 0;

    // whether or not the final padded word has already been decoded
    let mut reached_end = false;

    // loop through data
    loop {
        // request a new buffer from the reader
        let buffer = buf_reader.fill_buf()?;

        // exit loop if no more data
        let n = buffer.len();
        if n == 0 {
            break;
        }

        // for each byte in the buffer
        for &b in buffer {
            // decode the character and add to the word
            let decoded_value: u8 = REVERSE_ALPHABET[b as usize];
            word[word_index] = decoded_value;
            word_index += 1;

            // if full word try to decode
            if word_index == 4 {
                // if all bytes are valid (happy path)
                if (word[0] | word[1] | word[2] | word[3]) < 64 && !reached_end {
                    // decode and output word
                    write_buffer[write_index] = (word[0] << 2) | (word[1] >> 4);
                    write_buffer[write_index + 1] = (word[1] << 4) | (word[2] >> 2);
                    write_buffer[write_index + 2] = (word[2] << 6) | (word[3]);
                    write_index += 3;
                    word_index = 0;
                } else {
                    // clean out garbage and whitespace
                    let mut i = 0;
                    while i < word_index {
                        match word[i] {
                            254 => {
                                // whitespace: shift word data left to replace it
                                word.copy_within((i + 1)..4, i);
                                word_index -= 1;
                            }
                            255 => {
                                // garbage: either eliminate or error out depending on ignore_garbage flag
                                if ignore_garbage {
                                    word.copy_within((i + 1)..4, i);
                                    word_index -= 1;
                                } else {
                                    return Err(std::io::Error::new(
                                        ErrorKind::Other,
                                        "invalid input",
                                    ));
                                }
                            }
                            _ => {
                                // valid char or padding: keep it
                                i += 1;
                            }
                        }
                    }

                    // if still full, must be the final padded word
                    if word_index == 4 {
                        // if already process final word, then this is garbage, and if we were
                        // ignoring garbage we already would have returned, so this is an error
                        if reached_end {
                            return Err(std::io::Error::new(ErrorKind::Other, "invalid input"));
                        }

                        // we've reached the end
                        reached_end = true;
                        word_index = 0;

                        // output final bytes
                        if word[0] == 64 || word[1] == 64 || word[3] != 64 {
                            // if either of firt two chars are padding, or the last char isn't, that's garbage
                            return Err(std::io::Error::new(ErrorKind::Other, "invalid input"));
                        } else if word[2] == 64 {
                            // if two padding chars, output final byte
                            write_buffer[write_index] = (word[0] << 2) | (word[1] >> 4);
                            write_index += 1;
                        } else {
                            // if one padding char, output final two byte
                            write_buffer[write_index] = (word[0] << 2) | (word[1] >> 4);
                            write_buffer[write_index + 1] = (word[1] << 4) | (word[2] >> 2);
                            write_index += 2;
                        }

                        // if ignoring garbage, nothing after this matters to just return
                        if ignore_garbage {
                            writer.write_all(&write_buffer[0..write_index])?;
                            return Ok(());
                        }
                    }
                }
            }
        }

        // output write buffer
        writer.write_all(&write_buffer[0..write_index])?;
        write_index = 0;

        // inform reader that the bytes have been consumed
        buf_reader.consume(n);
    }

    // if there is extraneous data and won't be ignored
    if word_index != 0 && !ignore_garbage {
        // if any of it is garbage, return an error
        for i in 0..word_index {
            if word[i] == 255 {
                return Err(std::io::Error::new(ErrorKind::Other, "invalid input"));
            }
        }
    }

    Ok(())
}

///
/// Encode data in base-64
///
/// # Arguments
///
/// * `reader` - Data to encode
/// * `writer` - Writer to which encoded data will be written
/// * `wrap` - Column at which to wrap encoded data
///
pub fn b64_encode(
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
    let mut write_buffer: [u8; 87380] = [0; 87380];
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
        let mut i = 0;
        while (i + 2) < read_index {
            let c = &read_buffer[i..i + 3];
            write_buffer[write_index] = ALPHABET[(c[0] >> 2) as usize];
            write_buffer[write_index + 1] = ALPHABET[(((c[0] & 0x3) << 4) | (c[1] >> 4)) as usize];
            write_buffer[write_index + 2] = ALPHABET[(((c[1] & 0xF) << 2) | (c[2] >> 6)) as usize];
            write_buffer[write_index + 3] = ALPHABET[(c[2] & 0x3F) as usize];
            write_index += 4;
            i += 3;
        }

        // copy remaining data to front of buffer
        if i != read_index {
            read_buffer.copy_within(i..read_index, 0);
        }

        // reset read index
        read_index %= 3;

        // output base-64 characters
        current_col = wrapping_write(&write_buffer, write_index, wrap, current_col, writer)?;
        write_index = 0;
    }

    // process remaining data
    match read_index % 3 {
        0 => {}
        1 => {
            // output last byte as two data chars and two padding chars
            let a: u8 = read_buffer[0];
            write_buffer[0] = ALPHABET[(a >> 2) as usize];
            write_buffer[1] = ALPHABET[((a & 0x3) << 4) as usize];
            write_buffer[2] = '=' as u8;
            write_buffer[3] = '=' as u8;
            let _ = wrapping_write(&write_buffer, 4, wrap, current_col, writer)?;
        }
        2 => {
            // output last two bytes as three data chars and one padding char
            let (a, b) = (read_buffer[0], read_buffer[1]);
            write_buffer[0] = ALPHABET[(a >> 2) as usize];
            write_buffer[1] = ALPHABET[(((a & 0x3) << 4) | (b >> 4)) as usize];
            write_buffer[2] = ALPHABET[((b & 0xF) << 2) as usize];
            write_buffer[3] = '=' as u8;
            let _ = wrapping_write(&write_buffer, 4, wrap, current_col, writer)?;
        }
        _ => {
            unreachable!("impossible mod 3 value");
        }
    }

    // add a final newline, if wrapping is enabled
    if wrap.is_some() {
        writer.write_all(b"\n")?;
    }

    Ok(())
}
