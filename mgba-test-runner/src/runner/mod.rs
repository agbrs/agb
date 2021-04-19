use std::ffi::CStr;
use std::ffi::CString;

#[allow(
    non_upper_case_globals,
    dead_code,
    non_camel_case_types,
    non_snake_case
)]
mod bindings {
    include!("bindings.rs");
}

pub struct MGBA {
    mgba: *mut bindings::MGBA,
}

pub struct VideoBuffer {
    width: u32,
    height: u32,
    buffer: *mut u32,
}

impl VideoBuffer {
    pub fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    pub fn get_pixel(&self, x: u32, y: u32) -> u32 {
        let offset = (y * self.width + x) as isize;
        assert!(x < self.width, "x must be in range 0 to {}", self.width);
        assert!(y < self.height, "y must be in range 0 to {}", self.height);
        unsafe { *self.buffer.offset(offset) }
    }
}

impl MGBA {
    pub fn new(filename: &str) -> Self {
        unsafe { bindings::set_logger(Some(logger)) };
        let c_str = CString::new(filename).expect("should be able to make cstring from filename");
        MGBA {
            mgba: unsafe { bindings::new_runner(c_str.as_ptr() as *mut i8) },
        }
    }

    pub fn get_video_buffer(&self) -> VideoBuffer {
        let c_video_buffer = unsafe { bindings::get_video_buffer(self.mgba) };
        VideoBuffer {
            width: c_video_buffer.width,
            height: c_video_buffer.height,
            buffer: c_video_buffer.buffer,
        }
    }

    pub fn advance_frame(&mut self) {
        unsafe { bindings::advance_frame(self.mgba) }
    }
}

static mut CALLBACK: Option<Box<dyn Fn(&str)>> = None;

pub fn set_logger(x: Box<dyn Fn(&str)>) {
    unsafe {
        assert!(CALLBACK.is_none());
        CALLBACK = Some(x);
    }
}

pub fn clear_logger() {
    unsafe { CALLBACK = None }
}

extern "C" fn logger(c_str: *mut i8) {
    unsafe {
        if let Some(f) = &CALLBACK {
            f(CStr::from_ptr(c_str)
                .to_str()
                .expect("should be able to convert logging message to rust String"));
        }
    }
}

impl Drop for MGBA {
    fn drop(&mut self) {
        unsafe { bindings::free_runner(self.mgba) }
    }
}
