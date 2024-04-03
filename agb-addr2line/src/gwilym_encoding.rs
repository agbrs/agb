use std::{slice::ChunksExact, sync::OnceLock};

const ALPHABET: &[u8] = b"0123456789=ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz";

// pub fn encode_16(input: u16) -> [u8; 3] {
//     let input = input as usize;
//     [
//         ALPHABET[input >> (16 - 5)],
//         ALPHABET[(input >> (16 - 10)) & 0b11111],
//         ALPHABET[input & 0b111111],
//     ]
// }

// pub fn encode_32(input: u32) -> [u8; 6] {
//     let input = input as usize;
//     let output_16 = encode_16(input as u16);
//     [
//         ALPHABET[(input >> (32 - 5)) | 0b100000],
//         ALPHABET[(input >> (32 - 10)) & 0b11111],
//         ALPHABET[(input >> (32 - 16)) & 0b111111],
//         output_16[0],
//         output_16[1],
//         output_16[2],
//     ]
// }

pub fn gwilym_decode(input: &str) -> anyhow::Result<GwilymDecodeIter<'_>> {
    GwilymDecodeIter::new(input)
}

pub struct GwilymDecodeIter<'a> {
    chunks: ChunksExact<'a, u8>,
}

impl<'a> GwilymDecodeIter<'a> {
    fn new(input: &'a str) -> anyhow::Result<Self> {
        let Some((input, version)) = input.rsplit_once('v') else {
            anyhow::bail!("Does not contain version");
        };

        if version != "1" {
            anyhow::bail!("Only version 1 is supported");
        }

        if input.len() % 3 != 0 {
            anyhow::bail!("Input string must have length a multiple of 3");
        }

        Ok(Self {
            chunks: input.as_bytes().chunks_exact(3),
        })
    }
}

impl<'a> Iterator for GwilymDecodeIter<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(chunk) = self.chunks.next() else {
            return None;
        };

        let value = decode_chunk(chunk);
        if value & (1 << 17) != 0 {
            return Some(self.next().unwrap_or(0) | (value << 16));
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
