use std::{
    io::{Read, Seek, Write},
    sync::{Arc, Mutex},
};

use super::VFile;

pub struct Shared<V> {
    inner: Arc<Mutex<V>>,
}

impl<V> Clone for Shared<V> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<V> Shared<V> {
    pub fn new(v: V) -> Self {
        Self {
            inner: Arc::new(Mutex::new(v)),
        }
    }

    pub fn try_into_inner(self) -> Result<V, Self> {
        Arc::try_unwrap(self.inner)
            .map(|x| x.into_inner().unwrap())
            .map_err(|e| Self { inner: e })
    }
}

impl<V: Clone> Shared<V> {
    pub fn into_inner(self) -> V {
        Arc::try_unwrap(self.inner)
            .map(|x| x.into_inner().unwrap())
            .unwrap_or_else(|x| x.lock().unwrap().clone())
    }
}

impl<V: Write> Write for Shared<V> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.lock().unwrap().flush()
    }
}

impl<V: Read> Read for Shared<V> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.lock().unwrap().read(buf)
    }
}

impl<V: Seek> Seek for Shared<V> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.inner.lock().unwrap().seek(pos)
    }
}

impl<V: VFile> VFile for Shared<V> {}
