use std::{
    fs::File,
    io::{Read, Seek, Write},
};

use super::VFile;

pub struct FileBacked {
    file: File,
}

impl FileBacked {
    pub fn new(file: File) -> Self {
        Self { file }
    }

    pub fn into_inner(self) -> File {
        self.file
    }
}

impl Seek for FileBacked {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.file.seek(pos)
    }
}

impl Write for FileBacked {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}

impl Read for FileBacked {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.file.read(buf)
    }
}

impl VFile for FileBacked {}
