use std::{slice::ChunksExact, sync::OnceLock};

use thiserror::Error;

const ALPHABET: &[u8] = b"0123456789=ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz";

#[derive(Debug, Error)]
pub enum GwilymDecodeError {
    #[error("Does not contain version")]
    NoVersion,
    #[error("Only version 1 is supported")]
    WrongVersion,
    #[error("Input must be a multiple of 3 but have {0}")]
    LengthWrong(usize),
}

pub fn gwilym_decode(input: &str) -> Result<GwilymDecodeIter<'_>, GwilymDecodeError> {
    GwilymDecodeIter::new(input)
}

pub struct GwilymDecodeIter<'a> {
    chunks: ChunksExact<'a, u8>,
}

impl<'a> GwilymDecodeIter<'a> {
    fn new(input: &'a str) -> Result<Self, GwilymDecodeError> {
        let input = input
            .strip_prefix("https://agbrs.dev/crash#")
            .unwrap_or(input);

        let Some((input, version)) = input.rsplit_once('v') else {
            return Err(GwilymDecodeError::NoVersion);
        };

        if version != "1" {
            return Err(GwilymDecodeError::WrongVersion);
        }

        if input.len() % 3 != 0 {
            return Err(GwilymDecodeError::LengthWrong(input.len()));
        }

        Ok(Self {
            chunks: input.as_bytes().chunks_exact(3),
        })
    }
}

impl<'a> Iterator for GwilymDecodeIter<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        let chunk = self.chunks.next()?;

        let value = decode_chunk(chunk);
        if value & (1 << 16) != 0 {
            let upper_bits = value << 16;
            let lower_bits = self.next().unwrap_or(0) & 0xffff;

            return Some(upper_bits | lower_bits);
        }

        Some(value | 0x0800_0000)
    }
}

fn decode_chunk(chunk: &[u8]) -> u32 {
    let a = get_value_for_char(chunk[0]);
    let b = get_value_for_char(chunk[1]);
    let c = get_value_for_char(chunk[2]);

    (a << (16 - 5)) | (b << (16 - 10)) | c
}

fn get_value_for_char(input: u8) -> u32 {
    static REVERSE_ALHPABET: OnceLock<[u8; 128]> = OnceLock::new();

    REVERSE_ALHPABET.get_or_init(|| {
        let mut result = [0; 128];
        for (i, &c) in ALPHABET.iter().enumerate() {
            result[c as usize] = i as u8;
        }

        result
    })[input as usize] as u32
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fmt::Write;

    #[test]
    fn should_correctly_decode_16s() -> Result<(), GwilymDecodeError> {
        assert_eq!(
            &gwilym_decode("2QI65Q69306Kv1")?.collect::<Vec<_>>(),
            &[0x0800_16d3, 0x0800_315b, 0x0800_3243, 0x0800_0195]
        );

        Ok(())
    }

    fn encode_16(input: u16) -> [u8; 3] {
        let input = input as usize;
        [
            ALPHABET[input >> (16 - 5)],
            ALPHABET[(input >> (16 - 10)) & 0b11111],
            ALPHABET[input & 0b111111],
        ]
    }

    fn encode_32(input: u32) -> [u8; 6] {
        let input = input as usize;
        let output_lower_16 = encode_16(input as u16);
        let input_upper_16 = input >> 16;
        [
            ALPHABET[(input_upper_16 >> (16 - 5)) | (1 << 5)],
            ALPHABET[(input_upper_16 >> (16 - 10)) & 0b11111],
            ALPHABET[input_upper_16 & 0b111111],
            output_lower_16[0],
            output_lower_16[1],
            output_lower_16[2],
        ]
    }

    #[test]
    fn should_correctly_decode_16s_and_32s() -> Result<(), Box<dyn std::error::Error>> {
        let trace: &[u32] = &[
            0x0300_2990,
            0x0800_3289,
            0x0500_2993,
            0x3829_2910,
            0xffff_ffff,
            0x0000_0000,
        ];

        let mut result = String::new();
        for &ip in trace {
            if ip & 0xFFFF_0000 == 0x0800_0000 {
                let encoded = encode_16(ip as u16);
                let encoded_s = std::str::from_utf8(&encoded)?;
                write!(&mut result, "{encoded_s}")?
            } else {
                let encoded = encode_32(ip);
                let encoded_s = std::str::from_utf8(&encoded)?;
                write!(&mut result, "{encoded_s}")?
            }
        }

        write!(&mut result, "v1")?;

        assert_eq!(&gwilym_decode(&result)?.collect::<Vec<_>>(), trace);

        Ok(())
    }

    #[test]
    fn should_strip_the_agbrsdev_prefix() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            &gwilym_decode("https://agbrs.dev/crash#2QI65Q69306Kv1")?.collect::<Vec<_>>(),
            &[0x0800_16d3, 0x0800_315b, 0x0800_3243, 0x0800_0195]
        );

        Ok(())
    }
}
