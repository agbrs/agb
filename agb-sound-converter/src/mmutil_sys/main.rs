use ::libc;

#[no_mangle]
pub static mut target_system: libc::c_int = 0;
#[no_mangle]
pub static mut ignore_sflags: libc::c_uchar = 0;
#[no_mangle]
pub static mut PANNING_SEP: libc::c_int = 128;
