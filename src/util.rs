use std::io;
use std::io::Read;

/// Reads until the buffer is filled or the reader signals EOF
pub fn read_to_buf(buf: &mut [u8], rd: &mut Read) -> Result<(), io::Error> {
    let mut total = 0;
    while total < buf.len() {
        let count = rd.read(&mut buf[total..])?;
        if count == 0 {
            // Buffer not yet filled, but EOF reached
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "eof reached prematurely",
            ));
        }
        total += count;
    }

    Ok(())
}
