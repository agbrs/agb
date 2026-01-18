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

The block size is dynamic. It has size `max(erase_size, 128)` and must be a multiple of the write size.

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

| Offset | Size              | Field                                                                    |
| ------ | ----------------- | ------------------------------------------------------------------------ |
| 0      | 8                 | Standard header, next block is empty                                     |
| 8      | 1                 | Slot state                                                               |
| 9      | 1                 | Logical slot ID                                                          |
| 10     | 2                 | First data block of save                                                 |
| 12     | 2                 | First metadata data block (0xffff = none)                                |
| 14     | 2                 | Reserved (zeros)                                                         |
| 16     | 4                 | Generation (u32)                                                         |
| 20     | 4                 | Data checksum (crc32 of all block payloads in the chain)                 |
| 24     | 4                 | Data length (u32, total bytes of actual data)                            |
| 28     | 4                 | Metadata length (u32, total bytes of metadata including inline portion)  |
| 32     | 4                 | Metadata checksum (crc32 of all metadata bytes)                          |
| 36     | `block_size - 36` | Metadata start (continues in data blocks if first metadata block != none)

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

5. **Rebuild the free list**: Any block not a global header, slot header, valid save data chain, or valid metadata chain is considered free. Ghost slot data and metadata chains _are_ considered free.

# Reading

## Reading save data

To read save data from a given save slot:

1.  Read slot header, get first data block index
2.  Traverse chain, concatenating payloads
3.  Truncate to data_length (last block may have padding)
4.  Verify concatenated data against the data checksum

## Reading metadata

To read metadata from a given save slot:

1.  Read slot header, get metadata length, metadata checksum, and first metadata block index
2.  Read inline metadata from slot header (up to `block_size - 36` bytes or metadata_length, whichever is smaller)
3.  If first metadata block != 0xffff, traverse the metadata chain, concatenating payloads
4.  Truncate to metadata_length (last block may have padding)
5.  Verify concatenated metadata against the metadata checksum

# Writing

Saving data into slot `S`.

The physical slot that we write to will be the current physical slot that the current ghost slot occupies.

1. **Serialize save bytes into memory**: We need to know how long it is going to be.
2. **Compute the data checksum**: Calculate the crc32 of the save data.
3. **Write the data chain**: Allocate blocks from the free list and write the save data chain.
4. **Serialize metadata bytes into memory**: We need to know how long it is going to be.
5. **Compute the metadata checksum**: Calculate the crc32 of the metadata.
6. **Write the metadata chain** (if needed): If metadata exceeds the inline portion (`block_size - 36` bytes), allocate blocks and write the overflow to data blocks.
7. **Write the new slot header over the current ghost slot**:
   - state = VALID
   - logical ID = `S`
   - first data block = start of save data chain
   - first metadata block = start of metadata chain (or 0xffff if metadata fits inline)
   - generation = (current slot `S` generation) + 1 (or 1 if slot `S` was empty)
   - data length = total data length
   - data checksum = crc32 of all data bytes
   - metadata length = total metadata length
   - metadata checksum = crc32 of all metadata bytes
   - metadata start = first `block_size - 36` bytes of metadata
8. **Mark old slot as ghost**: Update the old slot `S`'s physical header block with:
   - state = GHOST
   - everything else the same
9. **Free the old ghost's data and metadata blocks**: Return them to the free list, otherwise we're going to run out of space.
