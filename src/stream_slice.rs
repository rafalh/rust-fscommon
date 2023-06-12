use crate::io;
use crate::io::prelude::*;
use core::cmp;

/// Stream wrapper for accessing limited segment of data from underlying file or device.
#[derive(Clone)]
pub struct StreamSlice<T: Seek> {
    inner: T,
    start_offset: u64,
    current_offset: u64,
    size: u64,
}

impl<T: Seek> StreamSlice<T> {
    /// Creates new `StreamSlice` from inner stream and offset range.
    ///
    /// `start_offset` is inclusive offset of the first accessible byte.
    /// `end_offset` is exclusive offset of the first non-accessible byte.
    /// `start_offset` must be lower or equal to `end_offset`.
    pub fn new(mut inner: T, start_offset: u64, end_offset: u64) -> io::Result<Self> {
        debug_assert!(end_offset >= start_offset);
        inner.seek(io::SeekFrom::Start(start_offset))?;
        let size = end_offset - start_offset;
        Ok(StreamSlice {
            start_offset,
            size,
            inner,
            current_offset: 0,
        })
    }

    /// Returns inner object
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T: Seek> Read for StreamSlice<T>
where
    T: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let max_read_size = cmp::min((self.size - self.current_offset) as usize, buf.len());
        let bytes_read = self.inner.read(&mut buf[..max_read_size])?;
        self.current_offset += bytes_read as u64;
        Ok(bytes_read)
    }
}

impl<T: Write + Seek> Write for StreamSlice<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let max_write_size = cmp::min((self.size - self.current_offset) as usize, buf.len());
        let bytes_written = self.inner.write(&buf[..max_write_size])?;
        self.current_offset += bytes_written as u64;
        Ok(bytes_written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<T: Seek> Seek for StreamSlice<T> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        let new_offset = match pos {
            io::SeekFrom::Current(x) => self.current_offset as i64 + x,
            io::SeekFrom::Start(x) => x as i64,
            io::SeekFrom::End(x) => self.size as i64 + x,
        };
        if new_offset < 0 || new_offset as u64 > self.size {
            Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid seek"))
        } else {
            self.inner
                .seek(io::SeekFrom::Start(self.start_offset + new_offset as u64))?;
            self.current_offset = new_offset as u64;
            Ok(self.current_offset)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let buf = "BeforeTest dataAfter".to_string().into_bytes();
        let cur = io::Cursor::new(buf);
        let mut stream = StreamSlice::new(cur, 6, 6 + 9).unwrap();

        let mut data = String::new();
        stream.read_to_string(&mut data).unwrap();
        assert_eq!(data, "Test data");

        stream.seek(io::SeekFrom::Start(5)).unwrap();
        let mut data = String::new();
        stream.read_to_string(&mut data).unwrap();
        assert_eq!(data, "data");

        stream.seek(io::SeekFrom::Start(5)).unwrap();
        stream.write_all("Rust".as_bytes()).unwrap();
        assert!(stream.write_all("X".as_bytes()).is_err());
        stream.seek(io::SeekFrom::Start(0)).unwrap();
        let mut data = String::new();
        stream.read_to_string(&mut data).unwrap();
        assert_eq!(data, "Test Rust");
    }
}
