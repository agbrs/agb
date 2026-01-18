# Saving and loading

Most games need to save the player's progress so they can continue where they left off.
The Game Boy Advance supports several types of save media, each with different characteristics.
`agb` provides a single, opinionated API over all of the different types of save media.

# API overview

The save system is built around the [`SaveSlotManager`](https://docs.rs/agb/latest/agb/save/struct.SaveSlotManager.html).
This gives you:

- Multiple save slots - like in a classic RPG where you can have multiple independent saves
- Corruption detection - if saving fails part way through, `agb` will load the most recent successful save
- Metadata separation - each save slot has a 'summary' which is loaded immediately.

# Why separate metadata and data?

The save system separates each slot into two parts, metadata and data.

### Metadata

Metadata is small information about a save that you want to display in a save slot selection screen e.g. the player's name, playtime, current level, amount of gold they have.
This is loaded into memory when the save system initializes, so you can display all slots without any additional reads from save media.

### Data

Data is the full game state - e.g. inventory, quest progress, world state.
This is only loaded when the player actually selects a slot, since it can be much larger and reading from save media (especially EEPROM and Flash) is slow.

This design means your save selection menu can show "Slot 1: Alice - Level 42 - 12:34" instantly, without having to load each save file to extract that information.

# How to use save slots

Save slots work like the save systems in classic RPGs:

1. New game: When starting a new game, let the player choose an empty slot (or overwrite an existing one)
2. Saving: Write to the player's chosen slot whenever they save
3. Loading: Show all slots with their metadata, let the player pick one, then load the full data

A typical game might use 3-4 slots.
More slots require more save media space for slot headers and metadata, so don't use more than you need.

If you don't want to use save slots, then set the number of slots to 1.
You'll still get the benefits of corruption prevention and won't need to worry about save slots.

# Save media types

Game Boy Advance cartridges can contain different types of save media:

| Media type  | Size    | Speed     | Notes                                  |
| ----------- | ------- | --------- | -------------------------------------- |
| SRAM        | 32 KiB  | Fast      | Battery-backed RAM, simplest to use    |
| EEPROM 512B | 512 B   | Slow      | Very limited space, cheap              |
| EEPROM 8K   | 8 KiB   | Slow      | Limited space, cheap                   |
| Flash 64K   | 64 KiB  | Read fast | Writes are slow, good for larger saves |
| Flash 128K  | 128 KiB | Read fast | Writes are slow, best for large saves  |

For most games, **SRAM** is the easiest choice if you need up to 32KB of save data.
If you need more space, use **Flash**.
**EEPROM** is mainly used because it's cheap, but the limited space makes it challenging to work with - EEPROM 512B in particular can barely fit a single save slot with minimal data.

If you are wanting to run your game on a flash cart (like the ez flash), then you should configure your game to use SRAM.
If you want to write your game to override a bootleg cartridge, then those use often use flash, and provide you with lots of space to store your save data.

# Setting up saving

First, define your save data structure using [`serde`](https://serde.rs/) for serialization.
If you want to display information about save slots (e.g., player name, playtime) without loading the full save, you can also define a metadata structure:

```rust
use serde::{Deserialize, Serialize};

/// Metadata shown in save slot selection screens (optional)
#[derive(Clone, Serialize, Deserialize)]
struct SaveMetadata {
    player_name: [u8; 8],
    play_time_minutes: u32,
}

/// The actual game save data
#[derive(Clone, Serialize, Deserialize)]
struct SaveData {
    level: u32,
    score: u32,
    inventory: [u16; 20],
}
```

# Initializing the save system

Initialize the save system by calling one of the `init_*` methods on `gba.save`.
This returns a [`SaveSlotManager`](https://docs.rs/agb/latest/agb/save/struct.SaveSlotManager.html) that you'll use for all save operations.

```rust
use agb::save::SaveSlotManager;

// A unique 32-byte identifier for your game.
// If this doesn't match what's stored, the save is considered incompatible
// and will be reformatted. Change this when your save format changes
// in a backwards-incompatible way.
//
// During development, it can be useful to set this to e.g. the current git
// commit hash. That way wheneven you change the format of the `Savemetadata`
// or `SaveData`, provided you commit your changes it'll automatically invalidate
// all existing save files.
const SAVE_MAGIC: [u8; 32] = *b"my-awesome-game-v1______________";

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    // Initialize SRAM with 3 save slots
    let mut save_manager: SaveSlotManager<SaveMetadata> = gba
        .save
        .init_sram(
            3,              // number of save slots
            SAVE_MAGIC,     // game identifier
        )
        .expect("Failed to initialize save");

    // Your game loop...
}
```

For Flash or EEPROM, you may want to provide a timer for timeout handling:

```rust
let timers = gba.timers.timers();
let mut save_manager: SaveSlotManager<SaveMetadata> = gba
    .save
    .init_flash_128k(3, SAVE_MAGIC, Some(timers.timer2))
    .expect("Failed to initialize save");
```

The available initialization methods are:

- `init_sram(num_slots, magic)` - For SRAM
- `init_eeprom_512b(num_slots, magic, timer)` - For 512B EEPROM
- `init_eeprom_8k(num_slots, magic, timer)` - For 8K EEPROM
- `init_flash_64k(num_slots, magic, timer)` - For 64K Flash
- `init_flash_128k(num_slots, magic, timer)` - For 128K Flash

# Writing save data

Use the [`write()`](https://docs.rs/agb/latest/agb/save/struct.SaveSlotManager.html#method.write) method to save data to a slot:

```rust
let save_data = SaveData {
    level: 5,
    score: 12500,
    inventory: [0; 20],
};

let metadata = SaveMetadata {
    player_name: *b"Player1\0",
    play_time_minutes: 45,
};

save_manager
    .write(0, &save_data, &metadata)  // write to slot 0
    .expect("Failed to save");
```

The save system is designed to be crash-safe.
If the game crashes or loses power during a write, the previous save data will be recovered on the next load.

# Reading save data

Use the [`read()`](https://docs.rs/agb/latest/agb/save/struct.SaveSlotManager.html#method.read) method to load save data:

```rust
let loaded: SaveData = save_manager
    .read(0)  // read from slot 0
    .expect("Failed to load save");
```

You can check the status of a slot before reading:

```rust
use agb::save::Slot;

match save_manager.slot(0) {
    Slot::Valid(metadata) => {
        let data: SaveData = save_manager.read(0).expect("Failed to load");
        // Use the loaded data...
    }
    Slot::Empty => {
        // No save data, start a new game
    }
    Slot::Corrupted => {
        // Save data is corrupted, cannot be recovered
        // This can happen if save media is damaged or
        // if a crash occurred and recovery failed
    }
}
```

# Displaying save slots

For a save slot selection screen, you can iterate over all slots:

```rust
use agb::save::Slot;

for slot in save_manager.slots() {
    match slot {
        Slot::Valid(metadata) => {
            // Display: "Player1 - 45 min"
            // metadata is available directly
        }
        Slot::Empty => {
            // Display: "Empty"
        }
        Slot::Corrupted => {
            // Display: "Corrupted"
        }
    }
}
```

The metadata is read during initialisation, so accessing it doesn't require reading from save media.

# Erasing a save slot

To delete a save slot:

```rust
save_manager.erase(0).expect("Failed to erase slot");
```

# Error handling

Save operations can fail for various reasons.
The [`SaveError`](https://docs.rs/agb_save/latest/agb_save/enum.SaveError.html) enum describes what went wrong:

- `SlotEmpty` - Tried to read from an empty slot
- `SlotCorrupted` - The slot data is corrupted
- `OutOfSpace` - Not enough free space for the write
- `SerializationFailed` - Failed to serialize the data
- `DeserializationFailed` - Failed to deserialize the data
- `Storage` - Low-level storage error

For a real game, you'd want to handle these gracefully:

```rust
use agb::save::SaveError;

match save_manager.read::<SaveData>(0) {
    Ok(data) => {
        // Use the loaded data
    }
    Err(SaveError::SlotEmpty) => {
        // Start a new game
    }
    Err(SaveError::SlotCorrupted) => {
        // Offer to start fresh or try another slot
    }
    Err(SaveError::OutOfSpace) => {
        // Save media is full - ask user to delete old saves
    }
    Err(e) => {
        // Handle other errors
        agb::println!("Save error: {:?}", e);
    }
}
```

# Space considerations

The save system has some overhead beyond your actual save data:

- A header sector for the magic identifier
- A slot header for each slot (stores metadata and data locations)
- Checksums for corruption detection

For most save media types this overhead is negligible, but for EEPROM 512B (only 512 bytes total), you may only be able to fit a single slot with very small data.
If you're targeting EEPROM 512B, you may not be able to store anything in data and put everything in metadata.
Use unit `()` as your data type.

# Save file upgrades

You may want to provide the ability for your save file to be upgraded to newer versions.
This is possible, but you have to do some additional work yourself to make it happen.

The `Metadata` type is fixed, and must be the same when loading data.
However, the `Data` type is not fixed.
Which `Data` type you deserialize from is up to you, and can depend on the metadata for that save slot.
So you could store the version in the `Metadata` type and then load a different save version depending on what you find there.

# Further reading

- [`SaveSlotManager` documentation](https://docs.rs/agb/latest/agb/save/struct.SaveSlotManager.html)
- [`SaveManager` documentation](https://docs.rs/agb/latest/agb/save/struct.SaveManager.html)
- [serde documentation](https://serde.rs/) for custom serialization
