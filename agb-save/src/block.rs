#[cfg(test)]
mod proptest;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BlockHeader {
    crc16: u16,
    block_type: BlockType,
    next_block_index: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BlockType {
    Free,
    Global,
    Slot,
    Data,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct GlobalHeader {
    slot_count: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SlotHeader {
    state: SlotState,
    logical_slot_id: u8,
    first_data_block: u16,
    generation: u32,
    crc32: u32,
    length: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DataBlockHeader {
    pub(crate) next_block: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SlotState {
    Empty,
    Valid,
    Ghost,
}

static LIBRARY_MAGIC: [u8; 4] = *b"agbS";

pub(crate) enum BlockLoadError {
    CrcMismatch,
    InvalidData,
}

fn crc16(data: &[u8]) -> u16 {
    crc16::State::<crc16::ARC>::calculate(data)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Block<'a> {
    Free,
    Global(GlobalBlock<'a>),
    SlotHeader(SlotHeaderBlock<'a>),
    Data(DataBlock<'a>),
}

/// Size of the standard block header (CRC16 + block type + next block + reserved)
pub const BLOCK_HEADER_SIZE: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlobalBlock<'a> {
    header: GlobalHeader,
    pub game_identifier: &'a [u8],
}

impl<'a> GlobalBlock<'a> {
    pub fn new(slot_count: u16, game_identifier: &'a [u8]) -> Self {
        Self {
            header: GlobalHeader { slot_count },
            game_identifier,
        }
    }

    pub fn slot_count(&self) -> u16 {
        self.header.slot_count
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlotHeaderBlock<'a> {
    header: SlotHeader,
    metadata: &'a [u8],
}

impl<'a> SlotHeaderBlock<'a> {
    /// Size of the slot header block header (standard header + slot header fields)
    /// Metadata starts at this offset.
    pub const fn header_size() -> usize {
        BLOCK_HEADER_SIZE + 16 // 8 + state(1) + logical_id(1) + first_block(2) + generation(4) + crc32(4) + length(4) = 24
    }

    /// Create an empty slot header for a given logical slot.
    pub(crate) fn empty(logical_slot_id: u8, metadata: &'a [u8]) -> Self {
        Self::empty_with_generation(logical_slot_id, 0, metadata)
    }

    /// Create an empty slot header with a specific generation.
    pub(crate) fn empty_with_generation(
        logical_slot_id: u8,
        generation: u32,
        metadata: &'a [u8],
    ) -> Self {
        Self {
            header: SlotHeader {
                state: SlotState::Empty,
                logical_slot_id,
                first_data_block: 0xFFFF,
                generation,
                crc32: 0,
                length: 0,
            },
            metadata,
        }
    }

    /// Create a ghost slot header (used as staging area).
    pub(crate) fn ghost(logical_slot_id: u8, metadata: &'a [u8]) -> Self {
        Self {
            header: SlotHeader {
                state: SlotState::Ghost,
                logical_slot_id,
                first_data_block: 0xFFFF,
                generation: 0,
                crc32: 0,
                length: 0,
            },
            metadata,
        }
    }

    /// Create a valid slot header with data.
    pub(crate) fn valid(
        logical_slot_id: u8,
        first_data_block: u16,
        generation: u32,
        crc32: u32,
        length: u32,
        metadata: &'a [u8],
    ) -> Self {
        Self {
            header: SlotHeader {
                state: SlotState::Valid,
                logical_slot_id,
                first_data_block,
                generation,
                crc32,
                length,
            },
            metadata,
        }
    }

    /// Create a slot header by changing only the state of this one.
    pub(crate) fn with_state(&self, state: SlotState) -> SlotHeaderBlock<'a> {
        SlotHeaderBlock {
            header: SlotHeader {
                state,
                ..self.header
            },
            metadata: self.metadata,
        }
    }

    pub(crate) fn state(&self) -> SlotState {
        self.header.state
    }

    pub(crate) fn logical_slot_id(&self) -> u8 {
        self.header.logical_slot_id
    }

    pub(crate) fn first_data_block(&self) -> u16 {
        self.header.first_data_block
    }

    pub(crate) fn generation(&self) -> u32 {
        self.header.generation
    }

    pub(crate) fn crc32(&self) -> u32 {
        self.header.crc32
    }

    pub(crate) fn length(&self) -> u32 {
        self.header.length
    }

    pub(crate) fn metadata(&self) -> &[u8] {
        self.metadata
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataBlock<'a> {
    pub(crate) header: DataBlockHeader,
    pub data: &'a [u8],
}

impl<'a> DataBlock<'a> {
    /// Size of the data block header (standard header only, next_block is in standard header)
    /// Data starts at this offset.
    pub const fn header_size() -> usize {
        BLOCK_HEADER_SIZE // 8
    }

    /// Create a new data block.
    ///
    /// - `next_block`: Index of the next block in the chain, or `0xFFFF` if this is the last block.
    /// - `data`: The payload data for this block.
    pub fn new(next_block: u16, data: &'a [u8]) -> Self {
        Self {
            header: DataBlockHeader { next_block },
            data,
        }
    }
}

pub fn deserialize_block(block_data: &[u8]) -> Result<Block<'_>, BlockLoadError> {
    let block_header = BlockHeader::try_from(block_data)?;
    let calculated_crc16 = crc16(&block_data[2..]);

    if calculated_crc16 != block_header.crc16 {
        return Err(BlockLoadError::CrcMismatch);
    }

    Ok(match block_header.block_type {
        BlockType::Free => Block::Free,
        BlockType::Global => Block::Global(GlobalBlock {
            header: GlobalHeader::try_from(&block_data[8..])?,
            game_identifier: &block_data[14..],
        }),
        BlockType::Slot => Block::SlotHeader(SlotHeaderBlock {
            header: SlotHeader::try_from(&block_data[8..])?,
            metadata: &block_data[24..],
        }),
        BlockType::Data => Block::Data(DataBlock {
            header: DataBlockHeader {
                next_block: block_header.next_block_index,
            },
            data: &block_data[8..],
        }),
    })
}

/// Writes the content of block into the buffer. Panics if the buffer is
/// the wrong length.
pub fn serialize_block(block: Block, buffer: &mut [u8]) {
    // build up the standard header
    buffer[2..4].copy_from_slice(&(block.kind() as u16).to_le_bytes());
    buffer[4..6].copy_from_slice(&match &block {
        Block::Data(data_block) => data_block.header.next_block.to_le_bytes(),
        _ => [0, 0],
    });
    buffer[6..8].copy_from_slice(&[0, 0]);

    match block {
        Block::Free => buffer[8..].iter_mut().for_each(|x| *x = 0),
        Block::Global(global_block) => {
            buffer[8..12].copy_from_slice(&LIBRARY_MAGIC);
            buffer[12..14].copy_from_slice(&global_block.header.slot_count.to_le_bytes());
            buffer[14..(14 + 32)].copy_from_slice(global_block.game_identifier);
        }
        Block::SlotHeader(slot_header_block) => {
            buffer[8] = slot_header_block.header.state as u8;
            buffer[9] = slot_header_block.header.logical_slot_id;
            buffer[10..12]
                .copy_from_slice(&slot_header_block.header.first_data_block.to_le_bytes());
            buffer[12..16].copy_from_slice(&slot_header_block.header.generation.to_le_bytes());
            buffer[16..20].copy_from_slice(&slot_header_block.header.crc32.to_le_bytes());
            buffer[20..24].copy_from_slice(&slot_header_block.header.length.to_le_bytes());
            buffer[24..].copy_from_slice(slot_header_block.metadata);
        }
        Block::Data(data_block) => {
            buffer[8..].copy_from_slice(data_block.data);
        }
    }

    // calculate the resulting crc16 for the header
    let checksum = crc16(&buffer[2..]);
    buffer[0..2].copy_from_slice(&checksum.to_le_bytes());
}

impl Block<'_> {
    fn kind(&self) -> BlockType {
        match self {
            Block::Free => BlockType::Free,
            Block::Global(_) => BlockType::Global,
            Block::SlotHeader(_) => BlockType::Slot,
            Block::Data(_) => BlockType::Data,
        }
    }
}

impl TryFrom<&[u8]> for BlockHeader {
    type Error = BlockLoadError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 6 {
            return Err(BlockLoadError::InvalidData);
        }

        Ok(Self {
            crc16: u16::from_le_bytes(value[0..2].try_into().unwrap()),
            block_type: value[2..4].try_into()?,
            next_block_index: u16::from_le_bytes(value[4..6].try_into().unwrap()),
        })
    }
}

impl TryFrom<&[u8]> for BlockType {
    type Error = BlockLoadError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != 2 {
            return Err(BlockLoadError::InvalidData);
        }

        Ok(match u16::from_le_bytes(value.try_into().unwrap()) {
            0 => Self::Free,
            1 => Self::Global,
            2 => Self::Slot,
            3 => Self::Data,
            _ => return Err(BlockLoadError::InvalidData),
        })
    }
}

impl TryFrom<&[u8]> for GlobalHeader {
    type Error = BlockLoadError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 6 {
            return Err(BlockLoadError::InvalidData);
        }

        let library_magic: [u8; 4] = value[..4].try_into().unwrap();
        if library_magic != LIBRARY_MAGIC {
            return Err(BlockLoadError::InvalidData);
        }

        let slot_count = u16::from_le_bytes(value[4..6].try_into().unwrap());
        Ok(Self { slot_count })
    }
}

impl TryFrom<&[u8]> for SlotHeader {
    type Error = BlockLoadError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 16 {
            return Err(BlockLoadError::InvalidData);
        }

        let slot_state = SlotState::try_from(value[0])?;
        let logical_slot_id = value[1];
        let first_data_block = u16::from_le_bytes(value[2..4].try_into().unwrap());
        let generation = u32::from_le_bytes(value[4..8].try_into().unwrap());
        let data_checksum = u32::from_le_bytes(value[8..12].try_into().unwrap());
        let data_length = u32::from_le_bytes(value[12..16].try_into().unwrap());

        Ok(Self {
            state: slot_state,
            logical_slot_id,
            first_data_block,
            generation,
            crc32: data_checksum,
            length: data_length,
        })
    }
}

impl TryFrom<u8> for SlotState {
    type Error = BlockLoadError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Empty,
            1 => Self::Valid,
            2 => Self::Ghost,
            _ => return Err(BlockLoadError::InvalidData),
        })
    }
}
