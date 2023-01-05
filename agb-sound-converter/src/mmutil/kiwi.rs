use ::libc;
extern "C" {
    fn printf(_: *const libc::c_char, _: ...) -> libc::c_int;
    fn rand() -> libc::c_int;
    fn srand(__seed: libc::c_uint);
    fn time(__timer: *mut time_t) -> time_t;
}
pub type u32_0 = libc::c_uint;
pub type __time_t = libc::c_long;
pub type time_t = __time_t;
#[no_mangle]
pub static mut time_start: u32_0 = 0;
#[no_mangle]
pub unsafe extern "C" fn kiwi_start() {
    let mut r: libc::c_int = 0;
    time_start = time(0 as *mut time_t) as u32_0;
    srand(time_start);
    r = 0 as libc::c_int;
    rand();
    rand();
    rand();
    rand();
    rand();
    rand();
    rand();
    rand();
    rand();
    rand();
    rand();
    rand();
    rand();
    rand();
    match r {
        0 => {
            printf(
                b"Your lucky number today is %i!\n\0" as *const u8 as *const libc::c_char,
                rand(),
            );
        }
        _ => {}
    };
}
