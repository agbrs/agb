use ::libc::{self, fclose, fopen, fread, fseek, ftell, fwrite, remove, FILE};

pub type u16_0 = libc::c_ushort;
pub type u32_0 = libc::c_uint;
pub type u8_0 = libc::c_uchar;
pub type bool_0 = libc::c_uchar;
pub type size_t = libc::c_ulong;
pub type __off_t = libc::c_long;
pub type __off64_t = libc::c_long;

#[no_mangle]
pub static mut fin: *mut FILE = 0 as *const FILE as *mut FILE;
#[no_mangle]
pub static mut fout: *mut FILE = 0 as *const FILE as *mut FILE;
#[no_mangle]
pub static mut file_byte_count: libc::c_int = 0;
#[no_mangle]
pub unsafe extern "C" fn file_exists(mut filename: *mut libc::c_char) -> bool_0 {
    fin = fopen(filename, b"rb\0" as *const u8 as *const libc::c_char);
    if fin.is_null() {
        return 0 as libc::c_int as bool_0;
    }
    file_close_read();
    return (0 as libc::c_int == 0) as libc::c_int as bool_0;
}
#[no_mangle]
pub unsafe extern "C" fn file_size(mut filename: *mut libc::c_char) -> libc::c_int {
    let mut f = 0 as *mut FILE;
    let mut a: libc::c_int = 0;
    f = fopen(filename, b"rb\0" as *const u8 as *const libc::c_char);
    if f.is_null() {
        return 0 as libc::c_int;
    }
    fseek(f, 0 as libc::c_int as libc::c_long, 2 as libc::c_int);
    a = ftell(f) as libc::c_int;
    fclose(f);
    return a;
}
#[no_mangle]
pub unsafe extern "C" fn file_open_read(mut filename: *mut libc::c_char) -> libc::c_int {
    fin = fopen(filename, b"rb\0" as *const u8 as *const libc::c_char);
    if fin.is_null() {
        return -(1 as libc::c_int);
    }
    return 0 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn file_open_write(mut filename: *mut libc::c_char) -> libc::c_int {
    fout = fopen(filename, b"wb\0" as *const u8 as *const libc::c_char);
    if fout.is_null() {
        return -(1 as libc::c_int);
    }
    return 0 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn file_open_write_end(mut filename: *mut libc::c_char) -> libc::c_int {
    fout = fopen(filename, b"r+b\0" as *const u8 as *const libc::c_char);
    if fout.is_null() {
        return -(1 as libc::c_int);
    }
    fseek(fout, 0 as libc::c_int as libc::c_long, 2 as libc::c_int);
    return 0 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn file_close_read() {
    fclose(fin);
}
#[no_mangle]
pub unsafe extern "C" fn file_close_write() {
    fclose(fout);
}
#[no_mangle]
pub unsafe extern "C" fn file_seek_read(
    mut offset: libc::c_int,
    mut mode: libc::c_int,
) -> libc::c_int {
    return fseek(fin, offset as libc::c_long, mode);
}
#[no_mangle]
pub unsafe extern "C" fn file_seek_write(
    mut offset: libc::c_int,
    mut mode: libc::c_int,
) -> libc::c_int {
    return fseek(fout, offset as libc::c_long, mode);
}
#[no_mangle]
pub unsafe extern "C" fn file_tell_read() -> libc::c_int {
    return ftell(fin) as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn file_tell_write() -> libc::c_int {
    return ftell(fout) as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn file_tell_size() -> libc::c_int {
    let mut pos = ftell(fin) as libc::c_int;
    fseek(fin, 0 as libc::c_int as libc::c_long, 2 as libc::c_int);
    let mut size = ftell(fin) as libc::c_int;
    fseek(fin, pos as libc::c_long, 0 as libc::c_int);
    return size;
}
#[no_mangle]
pub unsafe extern "C" fn read8() -> u8_0 {
    let mut a: u8_0 = 0;
    fread(&mut a as *mut u8_0 as *mut libc::c_void, 1, 1, fin);
    return a;
}
#[no_mangle]
pub unsafe extern "C" fn read16() -> u16_0 {
    let mut a: u16_0 = 0;
    a = read8() as u16_0;
    a = (a as libc::c_int | (read8() as u16_0 as libc::c_int) << 8 as libc::c_int) as u16_0;
    return a;
}
#[no_mangle]
pub unsafe extern "C" fn read24() -> u32_0 {
    let mut a: u32_0 = 0;
    a = read8() as u32_0;
    a |= (read8() as u32_0) << 8 as libc::c_int;
    a |= (read8() as u32_0) << 16 as libc::c_int;
    return a;
}
#[no_mangle]
pub unsafe extern "C" fn read32() -> u32_0 {
    let mut a: u32_0 = 0;
    a = read16() as u32_0;
    a |= (read16() as u32_0) << 16 as libc::c_int;
    return a;
}
#[no_mangle]
pub unsafe extern "C" fn read8f(mut p_fin: *mut FILE) -> u8_0 {
    let mut a: u8_0 = 0;
    fread(&mut a as *mut u8_0 as *mut libc::c_void, 1, 1, p_fin);
    return a;
}
#[no_mangle]
pub unsafe extern "C" fn read16f(mut p_fin: *mut FILE) -> u16_0 {
    let mut a: u16_0 = 0;
    a = read8f(p_fin) as u16_0;
    a = (a as libc::c_int | (read8f(p_fin) as u16_0 as libc::c_int) << 8 as libc::c_int) as u16_0;
    return a;
}
#[no_mangle]
pub unsafe extern "C" fn read32f(mut p_fin: *mut FILE) -> u32_0 {
    let mut a: u32_0 = 0;
    a = read16f(p_fin) as u32_0;
    a |= (read16f(p_fin) as u32_0) << 16 as libc::c_int;
    return a;
}
#[no_mangle]
pub unsafe extern "C" fn write8(mut p_v: u8_0) {
    fwrite(&mut p_v as *mut u8_0 as *const libc::c_void, 1, 1, fout);
    file_byte_count += 1;
}
#[no_mangle]
pub unsafe extern "C" fn write16(mut p_v: u16_0) {
    write8((p_v as libc::c_int & 0xff as libc::c_int) as u8_0);
    write8((p_v as libc::c_int >> 8 as libc::c_int) as u8_0);
    file_byte_count += 2 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn write24(mut p_v: u32_0) {
    write8((p_v & 0xff as libc::c_int as libc::c_uint) as u8_0);
    write8((p_v >> 8 as libc::c_int & 0xff as libc::c_int as libc::c_uint) as u8_0);
    write8((p_v >> 16 as libc::c_int & 0xff as libc::c_int as libc::c_uint) as u8_0);
    file_byte_count += 3 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn write32(mut p_v: u32_0) {
    write16((p_v & 0xffff as libc::c_int as libc::c_uint) as u16_0);
    write16((p_v >> 16 as libc::c_int) as u16_0);
    file_byte_count += 4 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn align16() {
    if ftell(fout) & 1 as libc::c_int as libc::c_long != 0 {
        write8(0xba as libc::c_int as u8_0);
    }
}
#[no_mangle]
pub unsafe extern "C" fn align32() {
    if ftell(fout) & 3 as libc::c_int as libc::c_long != 0 {
        write8(0xba as libc::c_int as u8_0);
    }
    if ftell(fout) & 3 as libc::c_int as libc::c_long != 0 {
        write8(0xba as libc::c_int as u8_0);
    }
    if ftell(fout) & 3 as libc::c_int as libc::c_long != 0 {
        write8(0xba as libc::c_int as u8_0);
    }
}
#[no_mangle]
pub unsafe extern "C" fn align32f(mut p_file: *mut FILE) {
    if ftell(p_file) & 3 as libc::c_int as libc::c_long != 0 {
        write8(0xba as libc::c_int as u8_0);
    }
    if ftell(p_file) & 3 as libc::c_int as libc::c_long != 0 {
        write8(0xba as libc::c_int as u8_0);
    }
    if ftell(p_file) & 3 as libc::c_int as libc::c_long != 0 {
        write8(0xba as libc::c_int as u8_0);
    }
}
#[no_mangle]
pub unsafe extern "C" fn skip8(mut count: u32_0) {
    fseek(fin, count as libc::c_long, 1 as libc::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn skip8f(mut count: u32_0, mut p_file: *mut FILE) {
    fseek(p_file, count as libc::c_long, 1 as libc::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn file_delete(mut filename: *mut libc::c_char) {
    if file_exists(filename) != 0 {
        remove(filename);
    }
}
#[no_mangle]
pub unsafe extern "C" fn file_get_byte_count() -> libc::c_int {
    let mut a = file_byte_count;
    file_byte_count = 0 as libc::c_int;
    return a;
}
