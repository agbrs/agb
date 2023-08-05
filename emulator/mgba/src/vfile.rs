use std::io::Read;
use std::io::Result;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;

pub mod file;
pub mod memory;
pub mod shared;

pub enum MapFlag {
    Read,
    Write,
}

pub trait VFile: Seek + Read + Write {
    fn readline(&mut self, buffer: &mut [u8]) -> Result<usize> {
        let mut byte = 0;
        while byte < buffer.len() - 1 {
            if self.read(&mut buffer[byte..byte + 1])? == 0 {
                break;
            }

            byte += 1;
            if buffer[byte - 1] == b'\n' {
                break;
            }
        }
        buffer[byte] = b'\0';
        Ok(byte)
    }
    fn map(&mut self, size: usize, _flag: MapFlag) -> Result<Box<[u8]>> {
        let position = self.stream_position()?;
        let data = vec![0; size];
        let mut data = data.into_boxed_slice();

        self.seek(SeekFrom::Start(0))?;
        match self.read_exact(&mut data) {
            Ok(_) => {}
            Err(err) => match err.kind() {
                std::io::ErrorKind::UnexpectedEof => {}
                _ => return Err(err),
            },
        }
        self.seek(SeekFrom::Start(position))?;

        Ok(data)
    }
    fn unmap(&mut self, data: Box<[u8]>) -> Result<()> {
        // assume map was created with write
        let position = self.stream_position()?;
        self.seek(SeekFrom::Start(0))?;
        self.write_all(&data)?;
        self.seek(SeekFrom::Start(position))?;

        Ok(())
    }
    fn truncate(&mut self, size: usize) -> Result<()> {
        let position = self.stream_position()?;
        let stream_length = self.seek(SeekFrom::End(0))?;

        if (size as u64) > stream_length {
            self.seek(SeekFrom::Start(position))?;
            return Ok(());
        }

        self.seek(SeekFrom::Start(size as u64))?;

        let bytes_to_write = stream_length - size as u64;
        let bytes: Vec<u8> = std::iter::repeat(0).take(bytes_to_write as usize).collect();
        self.write_all(&bytes)?;

        self.seek(SeekFrom::Start(position))?;

        Ok(())
    }
    fn size(&mut self) -> Result<usize> {
        let position = self.stream_position()?;
        let stream_length = self.seek(SeekFrom::End(0))?;
        self.seek(SeekFrom::Start(position))?;

        Ok(stream_length as usize)
    }
    fn sync(&mut self, buffer: &[u8]) -> Result<()> {
        let position = self.stream_position()?;

        self.seek(SeekFrom::Start(0))?;
        self.write_all(buffer)?;
        self.seek(SeekFrom::Start(position))?;

        Ok(())
    }
}

#[repr(C)]
struct VFileInner<V: VFile> {
    vfile: mgba_sys::VFile,
    file: V,
}

pub struct VFileAlloc<V: VFile>(Box<VFileInner<V>>);

impl<V: VFile> VFileAlloc<V> {
    pub fn new(f: V) -> Self {
        Self(Box::new(VFileInner {
            vfile: unsafe { vfile_extern::create_vfile::<V>() },
            file: f,
        }))
    }

    pub(crate) fn into_mgba(self) -> *mut mgba_sys::VFile {
        let f = Box::into_raw(self.0) as *mut VFileInner<V>;
        f.cast()
    }
}

mod vfile_extern {
    use std::io::SeekFrom;

    /// Safety: Must be part of a VFileInner
    pub unsafe fn create_vfile<V: super::VFile>() -> mgba_sys::VFile {
        mgba_sys::VFile {
            close: Some(close::<V>),
            seek: Some(seek::<V>),
            read: Some(read::<V>),
            readline: Some(readline::<V>),
            write: Some(write::<V>),
            map: Some(map::<V>),
            unmap: Some(unmap::<V>),
            truncate: Some(truncate::<V>),
            size: Some(size::<V>),
            sync: Some(sync::<V>),
        }
    }

    use mgba_sys::VFile;

    extern "C" fn close<V: super::VFile>(vf: *mut VFile) -> bool {
        drop(unsafe { Box::from_raw(vf.cast::<super::VFileInner<V>>()) });
        true
    }

    unsafe fn with_inner<V: super::VFile, F, T>(vf: *mut VFile, f: F) -> T
    where
        F: FnOnce(&mut dyn super::VFile) -> T,
    {
        let vf = vf.cast::<super::VFileInner<V>>();
        let vf = &mut *vf;
        f(&mut vf.file)
    }

    extern "C" fn seek<V: super::VFile>(
        vf: *mut VFile,
        offset: mgba_sys::off_t,
        whence: std::os::raw::c_int,
    ) -> mgba_sys::off_t {
        unsafe {
            with_inner::<V, _, _>(vf, |vf| {
                // casts required for windows compatability
                #[allow(clippy::useless_conversion)]
                let seek = match whence {
                    libc::SEEK_CUR => SeekFrom::Current(offset.into()),
                    libc::SEEK_SET => SeekFrom::Start(offset as u64),
                    libc::SEEK_END => SeekFrom::End(offset.into()),
                    _ => return -1,
                };
                vf.seek(seek).map(|x| x as mgba_sys::off_t).unwrap_or(-1)
            })
        }
    }

    extern "C" fn read<V: super::VFile>(
        vf: *mut VFile,
        buffer: *mut ::std::os::raw::c_void,
        size: usize,
    ) -> isize {
        unsafe {
            with_inner::<V, _, _>(vf, |vf| {
                vf.read(std::slice::from_raw_parts_mut(buffer.cast(), size))
                    .map(|x| x as isize)
                    .unwrap_or(-1)
            })
        }
    }

    extern "C" fn readline<V: super::VFile>(
        vf: *mut VFile,
        buffer: *mut ::std::os::raw::c_char,
        size: usize,
    ) -> isize {
        unsafe {
            with_inner::<V, _, _>(vf, |vf| {
                vf.readline(std::slice::from_raw_parts_mut(buffer.cast(), size))
                    .map(|x| x as isize)
                    .unwrap_or(-1)
            })
        }
    }

    extern "C" fn write<V: super::VFile>(
        vf: *mut VFile,
        buffer: *const ::std::os::raw::c_void,
        size: usize,
    ) -> isize {
        unsafe {
            with_inner::<V, _, _>(vf, |vf| {
                vf.write(std::slice::from_raw_parts(buffer.cast(), size))
                    .map(|x| x as isize)
                    .unwrap_or(-1)
            })
        }
    }

    extern "C" fn map<V: super::VFile>(
        vf: *mut VFile,
        size: usize,
        flags: ::std::os::raw::c_int,
    ) -> *mut ::std::os::raw::c_void {
        unsafe {
            with_inner::<V, _, _>(vf, |vf| {
                let map_type = match flags as u32 {
                    mgba_sys::MAP_WRITE => super::MapFlag::Write,
                    _ => super::MapFlag::Read,
                };
                vf.map(size, map_type)
                    .map(|x| Box::leak(x).as_mut_ptr().cast())
                    .unwrap_or(std::ptr::null_mut())
            })
        }
    }

    extern "C" fn unmap<V: super::VFile>(
        vf: *mut VFile,
        memory: *mut ::std::os::raw::c_void,
        size: usize,
    ) {
        unsafe {
            with_inner::<V, _, _>(vf, |vf| {
                let b = Box::from_raw(std::slice::from_raw_parts_mut(memory.cast::<u8>(), size));
                let _ = vf.unmap(b);
            })
        }
    }

    extern "C" fn truncate<V: super::VFile>(vf: *mut VFile, size: usize) {
        unsafe {
            let _ = with_inner::<V, _, _>(vf, |vf| vf.truncate(size));
        }
    }
    extern "C" fn size<V: super::VFile>(vf: *mut VFile) -> isize {
        unsafe { with_inner::<V, _, _>(vf, |vf| vf.size().map(|x| x as isize).unwrap_or(-1)) }
    }

    extern "C" fn sync<V: super::VFile>(
        vf: *mut VFile,
        buffer: *mut ::std::os::raw::c_void,
        size: usize,
    ) -> bool {
        unsafe {
            with_inner::<V, _, _>(vf, |vf| {
                vf.sync(std::slice::from_raw_parts(buffer.cast(), size))
                    .is_ok()
            })
        }
    }
}
