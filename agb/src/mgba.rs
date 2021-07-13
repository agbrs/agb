use crate::memory_mapped::{MemoryMapped, MemoryMapped1DArray};
use core::fmt::Write;

#[derive(Eq, PartialEq, Clone, Copy)]
#[allow(dead_code)]
pub enum DebugLevel {
    Fatal = 0,
    Error = 1,
    Warning = 2,
    Info = 3,
    Debug = 4,
}

const OUTPUT_STRING: MemoryMapped1DArray<u8, 256> =
    unsafe { MemoryMapped1DArray::new(0x04FF_F600) };
const DEBUG_ENABLE: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x04FF_F780) };

const ENABLE_HANDSHAKE_IN: u16 = 0xC0DE;
const ENABLE_HANDSHAKE_OUT: u16 = 0x1DEA;

const DEBUG_LEVEL: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x04FF_F700) };
const DEBUG_FLAG_CODE: u16 = 0x0100;

fn is_running_in_mgba() -> bool {
    DEBUG_ENABLE.set(ENABLE_HANDSHAKE_IN);
    DEBUG_ENABLE.get() == ENABLE_HANDSHAKE_OUT
}

const NUMBER_OF_CYCLES: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x04FF_F800) };

pub fn number_of_cycles_tagged(tag: u16) {
    NUMBER_OF_CYCLES.set(tag)
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

    pub fn print(
        &mut self,
        output: core::fmt::Arguments,
        level: DebugLevel,
    ) -> Result<(), core::fmt::Error> {
        write!(self, "{}", output)?;
        self.set_level(level);
        Ok(())
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
        let mut str_iter = s.bytes();
        while self.bytes_written < 255 {
            match str_iter.next() {
                Some(byte) => {
                    OUTPUT_STRING.set(self.bytes_written, byte);
                    self.bytes_written += 1;
                }
                None => return Ok(()),
            }
        }
        Ok(())
    }
}
