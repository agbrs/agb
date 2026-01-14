# Rationale

There are a few key requirements here:

1. Multiple save slots - this is such a classic gba thing that we definitely want to support it here
2. Corruption resistance - people often use cheap clone cartridges, so we should be able to handle
   the potential corruption of the data if writes fail.
3. Summary for each save slot. There should be 2 different save for each save. One which is a summary
   which is nice for e.g. showing the player name in each save slot, and the main one.
4. It should support all save formats that `agb` does. So the API should be based around what's available
   there.

Non goals:

1. Partial save reads and writes. You get everything or nothing.
2. Upgrading save files. Normally you stamp a cartridge and then send it out. Live updates don't happen.
3. Raw byte access. You access everything through serde.

# Format

Data is split into blocks. Each block has exactly the same header to simplify the implementation.

The block size is dynamic. It has size of `max(erase_size, N)`. `N` will end up being the maximal length of the metadata, so maybe is user configurable?

| Offset | Size             | Field                                              |
| ------ | ---------------- | -------------------------------------------------- |
| 0      | 2                | CRC16 covering bytes 2..end of block               |
| 2      | 2                | Block type                                         |
| 4      | 2                | Next block index (0xffff = end / none)             |
| 6      | 2                | Reserved (zeros)                                   |
| 8      | `block_size - 8` | payload (interpretation depends on the block type) |

## Block types

| Value | Type          |
| ----- | ------------- |
| 0     | Free / unused |
| 1     | Global header |
| 2     | Slot header   |
| 3     | Data block    |

## Global header block (type 1)

The global header's job is to ensure that we have a valid save file for this game.

### Block layout

| Offset | Size | Field                                                     |
| ------ | ---- | --------------------------------------------------------- |
| 0      | 8    | standard header, next block is unused                     |
| 8      | 4    | Library magic: `agbS` (0x61 0x67 0x62 0x53)               |
| 12     | 2    | Slot count (N, not including ghost)                       |
| 14     | 32   | Game identifier (user provided e.g. game name + git hash) |

## Slot header block (type 2)

These are located at blocks `1 + physical slot index`.
Because of how saves work, the physical slot index and the logical slot index are not necessarily the same.
These declare the save data, but the actual save file storage happens in data blocks.

We don't store the next block in the standard header, instead storing it in the main data because this allows the abstraction to load all blocks in a chain. And since the metadata is stored in the header block, we don't want to have to load them all at once.

### Block layout

| Offset | Size              | Field                                                    |
| ------ | ----------------- | -------------------------------------------------------- |
| 0      | 8                 | Standard header, next block is empty                     |
| 8      | 1                 | Slot state                                               |
| 9      | 1                 | Logical slot ID                                          |
| 10     | 2                 | First data block of save                                 |
| 12     | 4                 | Generation (u32)                                         |
| 16     | 4                 | Data checksum (crc32 of all block payloads in the chain) |
| 20     | 4                 | Data length (u32, total bytes of actual data)            |
| 24     | `block_size - 24` | Metadata (user defined)                                  |

### Slot states

| Value | State | Meaning                                                            |
| ----- | ----- | ------------------------------------------------------------------ |
| 0     | EMPTY | Slot has never been written to / has been erased                   |
| 1     | VALID | Slot contains valid save data (although this needs to be verified) |
| 2     | GHOST | Slot is the backup / staging slot                                  |

## Data block (type 3)

These blocks store the actual block data. They form a linked list in the standard block header.

### Block layout

| Offset | Size             | Field           |
| ------ | ---------------- | --------------- |
| 0      | 8                | Standard header |
| 8      | `block_size - 8` | data            |

# Initial load and verification

When reading a block, it is assumed that the CRC in the standard header is validated against the data within the block.

1. **Load global header**: Read block 0. If invalid, the storage is uninitialised or fully corrupted.
2. **Validate identity**: Check the library magic and the game identifier. If mismatching, then assume fully corrupt.
3. **Load slot headers**: Read blocks 1..N+1
4. **Identify corruption**: For each slot header that fails to load, or whose data chain has any failing block or the whole CRC doesn't match

   1. Mark as corrupt
   2. Check if the ghost was for this slot
   3. If ghost is valid and was for this slot, recover from ghost
   4. Otherwise, this slot is unrecoverable.

5. **Rebuild the free list**: Any block not a global header, slot header or valid save slots are considered free. Ghost slot chain _is_ considered free.

# Reading

To read from a given save slot:

1.  Read slot header, get first data block index
2.  Traverse chain, concatenating payloads
3.  Truncate to data_length (last block may have padding)
4.  Verify concatenated data against the data checksum

# Writing

Saving data into slot `S`.

1. **Serialize save bytes into memory**: We need to know how long it is going to be.
2. **Allocate blocks**: Using the free list, allocate enough blocks to store the data required.
3. **Write the data chain**: Write the data chain from the allocated blocks.
4. **Compute the checksum**: Calculate the crc32 of the data.
5. **Serialize metadata bytes into memory**: Assume we've dropped the save bytes at this point.
6. **Write the new slot header header**:
   - state = VALID
   - logical ID = `S`.
   - generation = (current slot `S` generation) + 1 (or 1 if slot `S` was empty)
7. **Mark old slot as ghost**: Update the old slot `S`'s header block with:
   - state = ghost
   - everything else the same
