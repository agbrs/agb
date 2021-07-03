use std::ffi::c_void;
use std::ffi::CStr;
use std::ffi::CString;

#[allow(
    non_upper_case_globals,
    dead_code,
    non_camel_case_types,
    non_snake_case
)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/runner-bindings.rs"));
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
    pub fn new(filename: &str) -> Result<Self, anyhow::Error> {
        let c_str = CString::new(filename).expect("should be able to make cstring from filename");
        let mgba = unsafe { bindings::new_runner(c_str.as_ptr() as *mut i8) };
        if mgba.is_null() {
            Err(anyhow::anyhow!("could not create core"))
        } else {
            Ok(MGBA { mgba })
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
    pub fn set_logger(&mut self, mut logger: impl FnMut(&str)) {
        unsafe {
            let callback = generate_c_callback(move |message: *mut i8| {
                logger(
                    CStr::from_ptr(message)
                        .to_str()
                        .expect("should be able to convert logging message to rust String"),
                );
            });
            bindings::set_logger(self.mgba, callback)
        }
    }
}

unsafe fn generate_c_callback<F>(f: F) -> bindings::callback
where
    F: FnMut(*mut i8),
{
    let data = Box::into_raw(Box::new(f));

    bindings::callback {
        callback: Some(call_closure::<F>),
        data: data as *mut _,
        destroy: Some(drop_box::<F>),
    }
}

extern "C" fn call_closure<F>(data: *mut c_void, message: *mut i8)
where
    F: FnMut(*mut i8),
{
    let callback_ptr = data as *mut F;
    let callback = unsafe { &mut *callback_ptr };
    callback(message);
}

extern "C" fn drop_box<T>(data: *mut c_void) {
    unsafe {
        Box::from_raw(data as *mut T);
    }
}

impl Drop for MGBA {
    fn drop(&mut self) {
        unsafe { bindings::free_runner(self.mgba) }
    }
}
