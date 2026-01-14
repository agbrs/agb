extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use quickcheck::{Arbitrary, Gen, quickcheck};

use crate::block::{
    Block, BlockLoadError, BlockType, DataBlock, DataBlockHeader, GlobalBlock, GlobalHeader,
    SlotHeader, SlotHeaderBlock, SlotState, deserialize_block, serialize_block,
};

const TEST_BLOCK_SIZE: usize = 128;

impl Arbitrary for BlockType {
    fn arbitrary(g: &mut Gen) -> Self {
        *g.choose(&[Self::Data, Self::Free, Self::Global, Self::Slot])
            .unwrap()
    }
}

impl Arbitrary for SlotState {
    fn arbitrary(g: &mut Gen) -> Self {
        *g.choose(&[Self::Empty, Self::Valid, Self::Ghost]).unwrap()
    }
}

impl Arbitrary for GlobalHeader {
    fn arbitrary(g: &mut Gen) -> Self {
        Self {
            slot_count: u16::arbitrary(g),
        }
    }
}

impl Arbitrary for SlotHeader {
    fn arbitrary(g: &mut Gen) -> Self {
        Self {
            state: SlotState::arbitrary(g),
            logical_slot_id: u8::arbitrary(g),
            first_data_block: u16::arbitrary(g),
            generation: u32::arbitrary(g),
            crc32: u32::arbitrary(g),
            length: u32::arbitrary(g),
        }
    }
}

impl Arbitrary for DataBlockHeader {
    fn arbitrary(g: &mut Gen) -> Self {
        Self {
            next_block: u16::arbitrary(g),
        }
    }
}

/// Owned version of Block for testing, since Block contains references
#[derive(Clone, Debug, PartialEq, Eq)]
enum OwnedBlock {
    Free,
    Global {
        header: GlobalHeader,
        game_identifier: [u8; 32],
    },
    SlotHeader {
        header: SlotHeader,
        metadata: Vec<u8>,
    },
    Data {
        header: DataBlockHeader,
        data: Vec<u8>,
    },
}

impl Arbitrary for OwnedBlock {
    fn arbitrary(g: &mut Gen) -> Self {
        match BlockType::arbitrary(g) {
            BlockType::Free => OwnedBlock::Free,
            BlockType::Global => {
                let mut game_identifier = [0u8; 32];
                for byte in &mut game_identifier {
                    *byte = u8::arbitrary(g);
                }
                OwnedBlock::Global {
                    header: GlobalHeader::arbitrary(g),
                    game_identifier,
                }
            }
            BlockType::Slot => {
                // metadata size is block_size - 24 (8 byte standard header + 16 byte slot header)
                let metadata_size = TEST_BLOCK_SIZE - 24;
                let mut metadata = Vec::with_capacity(metadata_size);
                for _ in 0..metadata_size {
                    metadata.push(u8::arbitrary(g));
                }
                OwnedBlock::SlotHeader {
                    header: SlotHeader::arbitrary(g),
                    metadata,
                }
            }
            BlockType::Data => {
                // data size is block_size - 8
                let data_size = TEST_BLOCK_SIZE - 8;
                let data: Vec<u8> = (0..data_size).map(|_| u8::arbitrary(g)).collect();
                OwnedBlock::Data {
                    header: DataBlockHeader::arbitrary(g),
                    data,
                }
            }
        }
    }
}

impl OwnedBlock {
    fn to_block(&self) -> Block<'_> {
        match self {
            OwnedBlock::Free => Block::Free,
            OwnedBlock::Global {
                header,
                game_identifier,
            } => Block::Global(GlobalBlock {
                header: header.clone(),
                game_identifier,
            }),
            OwnedBlock::SlotHeader { header, metadata } => Block::SlotHeader(SlotHeaderBlock {
                header: header.clone(),
                metadata,
            }),
            OwnedBlock::Data { header, data } => Block::Data(DataBlock {
                header: header.clone(),
                data,
            }),
        }
    }

    fn from_block(block: &Block<'_>) -> Self {
        match block {
            Block::Free => OwnedBlock::Free,
            Block::Global(g) => {
                // game_identifier is stored as exactly 32 bytes, but deserialization
                // returns the rest of the block - we only care about the first 32
                let mut game_identifier = [0u8; 32];
                game_identifier.copy_from_slice(&g.game_identifier[..32]);
                OwnedBlock::Global {
                    header: g.header.clone(),
                    game_identifier,
                }
            }
            Block::SlotHeader(s) => OwnedBlock::SlotHeader {
                header: s.header.clone(),
                metadata: s.metadata.to_vec(),
            },
            Block::Data(d) => OwnedBlock::Data {
                header: d.header.clone(),
                data: d.data.to_vec(),
            },
        }
    }
}

quickcheck! {
    fn block_roundtrip(block: OwnedBlock) -> bool {
        let mut buffer = [0u8; TEST_BLOCK_SIZE];

        serialize_block(block.to_block(), &mut buffer);

        match deserialize_block(&buffer) {
            Ok(deserialized) => {
                let roundtripped = OwnedBlock::from_block(&deserialized);
                block == roundtripped
            }
            Err(_) => false,
        }
    }

    fn free_block_roundtrip() -> bool {
        let mut buffer = [0u8; TEST_BLOCK_SIZE];
        let block = Block::Free;

        serialize_block(block, &mut buffer);

        matches!(deserialize_block(&buffer), Ok(Block::Free))
    }

    fn global_block_roundtrip(slot_count: u16, game_id: Vec<u8>) -> bool {
        let mut game_identifier = [0u8; 32];
        for (i, &byte) in game_id.iter().take(32).enumerate() {
            game_identifier[i] = byte;
        }

        let mut buffer = [0u8; TEST_BLOCK_SIZE];
        let block = Block::Global(GlobalBlock {
            header: GlobalHeader { slot_count },
            game_identifier: &game_identifier,
        });

        serialize_block(block, &mut buffer);

        match deserialize_block(&buffer) {
            Ok(Block::Global(g)) => {
                // game_identifier in deserialized block extends to end of buffer,
                // but only first 32 bytes are meaningful
                g.header.slot_count == slot_count && g.game_identifier[..32] == game_identifier
            }
            _ => false,
        }
    }

    fn data_block_roundtrip(next_block: u16, data: Vec<u8>) -> bool {
        let data_size = TEST_BLOCK_SIZE - 8;
        let mut padded_data = vec![0u8; data_size];
        for (i, &byte) in data.iter().take(data_size).enumerate() {
            padded_data[i] = byte;
        }

        let mut buffer = [0u8; TEST_BLOCK_SIZE];
        let block = Block::Data(DataBlock {
            header: DataBlockHeader { next_block },
            data: &padded_data,
        });

        serialize_block(block, &mut buffer);

        match deserialize_block(&buffer) {
            Ok(Block::Data(d)) => {
                d.header.next_block == next_block && d.data == padded_data.as_slice()
            }
            _ => false,
        }
    }

    fn slot_header_roundtrip(header: SlotHeader, metadata_seed: Vec<u8>) -> bool {
        // metadata size is block_size - 24 (8 byte standard header + 16 byte slot header)
        let metadata_size = TEST_BLOCK_SIZE - 24;
        let mut padded_metadata = vec![0u8; metadata_size];
        for (i, &byte) in metadata_seed.iter().take(metadata_size).enumerate() {
            padded_metadata[i] = byte;
        }

        let mut buffer = [0u8; TEST_BLOCK_SIZE];
        let block = Block::SlotHeader(SlotHeaderBlock {
            header: header.clone(),
            metadata: &padded_metadata,
        });

        serialize_block(block, &mut buffer);

        match deserialize_block(&buffer) {
            Ok(Block::SlotHeader(s)) => {
                s.header == header && s.metadata == padded_metadata.as_slice()
            }
            _ => false,
        }
    }

    /// Modifying any byte after the CRC (bytes 2+) should cause deserialization to fail
    /// (either CrcMismatch if the CRC check catches it, or InvalidData if parsing fails first)
    fn corrupted_byte_detected(block: OwnedBlock, corrupt_offset: usize, corrupt_xor: u8) -> bool {
        let mut buffer = [0u8; TEST_BLOCK_SIZE];
        serialize_block(block.to_block(), &mut buffer);

        // Only corrupt bytes after the CRC (which is at bytes 0-1)
        // and ensure we actually change the byte (xor with non-zero)
        let offset = 2 + (corrupt_offset % (TEST_BLOCK_SIZE - 2));
        let xor_value = if corrupt_xor == 0 { 1 } else { corrupt_xor };

        buffer[offset] ^= xor_value;

        // Corruption should be detected - either as CRC mismatch or invalid data
        deserialize_block(&buffer).is_err()
    }

    /// Modifying the CRC itself should also cause a mismatch (unless we get very unlucky)
    fn corrupted_crc_detected(block: OwnedBlock, corrupt_xor: u8) -> bool {
        let mut buffer = [0u8; TEST_BLOCK_SIZE];
        serialize_block(block.to_block(), &mut buffer);

        // Corrupt the CRC bytes (first 2 bytes)
        let xor_value = if corrupt_xor == 0 { 1 } else { corrupt_xor };
        buffer[0] ^= xor_value;

        matches!(deserialize_block(&buffer), Err(BlockLoadError::CrcMismatch))
    }
}
