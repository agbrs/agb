use crate::memory_mapped::MemoryMapped;

#[derive(Eq, PartialEq, Clone, Copy)]
#[repr(u16)]
#[allow(dead_code)]
pub enum DebugLevel {
    Fatal = 0,
    Error = 1,
    Warning = 2,
    Info = 3,
    Debug = 4,
}

const OUTPUT: *mut u8 = 0x04FF_F600 as *mut u8;
const ENABLE: MemoryMapped<u16> = MemoryMapped::new(0x04FF_F780);

const ENABLE_HANDSHAKE_IN: u16 = 0xC0DE;
const ENABLE_HANDSHAKE_OUT: u16 = 0x1DEA;

const DEBUG_LEVEL: MemoryMapped<u16> = MemoryMapped::new(0x04FF_F700);
const DEBUG_FLAG_CODE: u16 = 0x0100;

fn is_running_in_mgba() -> bool {
    ENABLE.set(ENABLE_HANDSHAKE_IN);
    ENABLE.get() == ENABLE_HANDSHAKE_OUT
}

pub struct Mgba {
    bytes_written: usize,
}

impl Mgba {
    pub fn new() -> Option<Self> {
        if is_running_in_mgba() {
            Some(Mgba { bytes_written: 0 })
        } else {
            None
        }
    }
}

impl Mgba {
    pub fn set_level(&mut self, level: DebugLevel) {
        DEBUG_LEVEL.set(DEBUG_FLAG_CODE | level as u16);
        self.bytes_written = 0;
    }
}

impl core::fmt::Write for Mgba {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        unsafe {
            let mut current_location = OUTPUT.add(self.bytes_written);
            let mut str_iter = s.bytes();
            while self.bytes_written < 255 {
                match str_iter.next() {
                    Some(byte) => {
                        current_location.write(byte);
                        current_location = current_location.offset(1);
                        self.bytes_written += 1;
                    }
                    None => return Ok(()),
                }
            }
        }
        Ok(())
    }
}
