use alloc::vec::Vec;
use hashbrown::HashMap;

use crate::memory_mapped::MemoryMapped1DArray;

const TILE_BACKGROUND: MemoryMapped1DArray<u32, { 2048 * 8 }> =
    unsafe { MemoryMapped1DArray::new(0x06000000) };

#[derive(Clone, Copy, Debug)]
enum TileFormat {
    FourBpp,
}

impl TileFormat {
    /// Returns the size of the tile in bytes
    fn tile_size(self) -> usize {
        match self {
            TileFormat::FourBpp => 8 * 8 / 2,
        }
    }
}

struct TileSet<'a> {
    tiles: &'a [u8],
    format: TileFormat,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct TileSetReference {
    id: u16,
    generation: u32,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
struct TileIndex(u16);

enum ArenaStorageItem<T> {
    EndOfFreeList,
    NextFree(usize),
    Data(T, u32),
}

struct VRamManager<'a> {
    tilesets: Vec<ArenaStorageItem<TileSet<'a>>>,
    generation: u32,
    free_pointer: Option<usize>,

    tile_set_to_vram: HashMap<(u16, u16), u16>,
    references: Vec<u16>,
    vram_free_pointer: Option<usize>,
}

const END_OF_FREE_LIST_REFERENCE: u16 = u16::MAX;

impl<'a> VRamManager<'a> {
    pub fn new() -> Self {
        Self {
            tilesets: Vec::new(),
            generation: 0,
            free_pointer: None,

            tile_set_to_vram: HashMap::new(),
            references: Vec::new(),
            vram_free_pointer: None,
        }
    }

    pub fn add_tileset(&mut self, tileset: TileSet<'a>) -> TileSetReference {
        let generation = self.generation;
        self.generation += 1;

        let tileset = ArenaStorageItem::Data(tileset, generation);

        let index = if let Some(ptr) = self.free_pointer.take() {
            match self.tilesets[ptr] {
                ArenaStorageItem::EndOfFreeList => {
                    self.tilesets[ptr] = tileset;
                    ptr
                }
                ArenaStorageItem::NextFree(next_free) => {
                    self.free_pointer = Some(next_free);
                    self.tilesets[ptr] = tileset;
                    ptr
                }
                _ => panic!("Free pointer shouldn't point to valid data"),
            }
        } else {
            self.tilesets.push(tileset);
            self.tilesets.len() - 1
        };

        TileSetReference {
            id: index as u16,
            generation,
        }
    }

    pub fn remove_tileset(&mut self, tile_set_ref: TileSetReference) {
        let tileset = self.tilesets[tile_set_ref.id as usize];

        match tileset {
            ArenaStorageItem::Data(_, generation) => {
                assert_eq!(
                    generation, tile_set_ref.generation,
                    "Tileset generation must be the same when removing"
                );

                self.tilesets[tile_set_ref.id as usize] = if let Some(ptr) = self.free_pointer {
                    ArenaStorageItem::NextFree(ptr)
                } else {
                    ArenaStorageItem::EndOfFreeList
                };

                self.free_pointer = Some(tile_set_ref.id as usize);
            }
            _ => panic!("Already freed, probably a double free?"),
        }
    }

    pub fn add_tile(&mut self, tile_set_ref: TileSetReference, tile: u16) -> TileIndex {
        if let Some(&reference) = self.tile_set_to_vram.get(&(tile_set_ref.id, tile)) {
            self.references[reference as usize] += 1;
            return TileIndex(reference as u16);
        }

        let index_to_copy_into = if let Some(ptr) = self.vram_free_pointer.take() {
            if self.references[ptr] != END_OF_FREE_LIST_REFERENCE {
                self.vram_free_pointer = Some(self.references[ptr] as usize);
            }

            self.references[ptr] = 1;
            ptr
        } else {
            self.references.push(1);
            self.references.len() - 1
        };

        let tile_slice = if let ArenaStorageItem::Data(data, generation) =
            self.tilesets[tile_set_ref.id as usize]
        {
            assert_eq!(
                generation, tile_set_ref.generation,
                "Stale tile data requested"
            );

            let tile_offset = (tile as usize) * data.format.tile_size();
            &data.tiles[tile_offset..(tile_offset + data.format.tile_size())]
        } else {
            panic!("Cannot find tile data at given reference");
        };

        let tile_size_in_words = TileFormat::FourBpp.tile_size() / 4;

        unsafe {
            let (_, tile_data, _) = tile_slice.align_to::<u32>();

            for (i, &word) in tile_data.iter().enumerate() {
                TILE_BACKGROUND.set(index_to_copy_into * tile_size_in_words + i, word);
            }
        }

        TileIndex(index_to_copy_into as u16)
    }

    pub fn remove_tile(&mut self, tile_index: TileIndex) {
        let index = tile_index.0 as usize;
        self.references[index] -= 1;

        if self.references[index] != 0 {
            return;
        }

        if let Some(ptr) = self.vram_free_pointer {
            self.references[index] = ptr as u16;
        } else {
            self.references[index] = END_OF_FREE_LIST_REFERENCE;
        }

        self.vram_free_pointer = Some(index);
    }
}

impl TileSetReference {
    fn new(id: u16, generation: u32) -> Self {
        Self { id, generation }
    }
}
