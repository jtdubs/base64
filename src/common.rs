use std::io::Write;

pub fn wrapping_write(
    buffer: &[u8],
    len: usize,
    wrap_col: Option<usize>,
    mut current_col: usize,
    writer: &mut impl Write,
) -> Result<usize, std::io::Error> {
    // if wrapping is required
    if let Some(line_length) = wrap_col {
        let mut written: usize = 0;

        // while there are more bytes to write
        while written < len {
            // calculate bytes remaining in line and total bytes remaining
            let line_remaining: usize = line_length - current_col;
            let byte_remaining: usize = len - written;

            // bytes to write this iteration is the min of those values
            let n: usize = line_remaining.min(byte_remaining);

            // write the output
            writer.write_all(&buffer[written..written + n])?;
            written += n;

            // if we wrote all remaining bytes on this line
            if n == byte_remaining {
                // advance to the new column
                current_col += n;
                break;
            // otherwise
            } else {
                // add a newline and reset the column counter
                writer.write_all(b"\n")?;
                current_col = 0 as usize;
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
