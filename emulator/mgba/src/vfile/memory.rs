use std::{
    borrow::Cow,
    io::{Cursor, Read, Seek, Write},
};

use super::VFile;

pub struct MemoryBacked {
    buffer: Cursor<Cow<'static, [u8]>>,
}

impl MemoryBacked {
    pub fn new_from_slice(data: &'static [u8]) -> Self {
        Self {
            buffer: Cursor::new(Cow::Borrowed(data)),
        }
    }

    pub fn new(data: Vec<u8>) -> Self {
        Self {
            buffer: Cursor::new(Cow::Owned(data)),
        }
    }

    pub fn into_inner(self) -> Cow<'static, [u8]> {
        self.buffer.into_inner()
    }
}

impl Write for MemoryBacked {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let position = self.buffer.position();
        let underlying = self.buffer.get_mut().to_mut();
        let mut new_buffer = Cursor::new(underlying);
        new_buffer.set_position(position);
        let result = new_buffer.write(buf);
        let new_position = new_buffer.position();

        self.buffer.set_position(new_position);

        result
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Read for MemoryBacked {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.buffer.read(buf)
    }
}

impl Seek for MemoryBacked {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.buffer.seek(pos)
    }
}

impl VFile for MemoryBacked {}
