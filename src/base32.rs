use std::io::{BufRead, BufReader, ErrorKind, Read, Write};

use crate::common::wrapping_write;

// the canonical base-64 alphabet
const ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

// reverse lookup that maps:
// - base-32 chars back to their values (0-31)
// - base-32 padding to 32
// - whitespace to 254
// - garbage to 255
const REVERSE_ALPHABET: [u8; 256] = [
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x20, 0xFF, 0xFF,
    0xFF, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
    0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
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
/// Decode base-32 encoded data
///
/// # Arguments
///
/// * `reader` - Base-32 encoded data reader
/// * `writer` - Writer to which decoded data will be written
/// * `ignore_garbage` - Whether or not invalid characters should be ignored
///
pub fn b32_decode(
    reader: &mut impl Read,
    writer: &mut impl Write,
    ignore_garbage: bool,
) -> Result<(), std::io::Error> {
    // wrap the reader in a 64KiB buffered reader
    let mut buf_reader = BufReader::with_capacity(65536, reader);

    // internal write buffering to minimize calls to the writer
    let mut write_buffer: [u8; 65536] = [0; 65536];
    let mut write_index: usize = 0;

    // temp buffer for 8-char base-32 `word` to be decoded
    let mut word: [u8; 8] = [0; 8];
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
            if word_index == 8 {
                // if all bytes are valid (happy path)
                if (word[0] | word[1] | word[2] | word[3] | word[4] | word[5] | word[6] | word[7]) < 32 && !reached_end {
                    // decode and output word
                    write_buffer[write_index] = (word[0] << 3) | (word[1] >> 2);
                    write_buffer[write_index + 1] = (word[1] << 6) | (word[2] << 1) | (word[3] >> 4);
                    write_buffer[write_index + 2] = (word[3] << 4) | (word[4] >> 1);
                    write_buffer[write_index + 3] = (word[4] << 7) | (word[5] << 2) | (word[6] >> 3);
                    write_buffer[write_index + 4] = (word[6] << 5) | word[7];
                    write_index += 5;
                    word_index = 0;
                } else {
                    // clean out garbage and whitespace
                    let mut i = 0;
                    while i < word_index {
                        match word[i] {
                            254 => {
                                // whitespace: shift word data left to replace it
                                word.copy_within((i + 1)..8, i);
                                word_index -= 1;
                            }
                            255 => {
                                // garbage: either eliminate or error out depending on ignore_garbage flag
                                if ignore_garbage {
                                    word.copy_within((i + 1)..8, i);
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
                    if word_index == 8 {
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
