use core::{alloc::Layout, ptr::NonNull};

use alloc::vec;
use alloc::vec::Vec;

use crate::{
    agb_alloc::{block_allocator::BlockAllocator, bump_allocator::StartEnd},
    display::palette16,
    dma::dma_copy16,
    memory_mapped::MemoryMapped1DArray,
};

const TILE_RAM_START: usize = 0x0600_0000;

const PALETTE_BACKGROUND: MemoryMapped1DArray<u16, 256> =
    unsafe { MemoryMapped1DArray::new(0x0500_0000) };

static TILE_ALLOCATOR: BlockAllocator = unsafe {
    BlockAllocator::new(StartEnd {
        start: || TILE_RAM_START,
        end: || TILE_RAM_START + 0x8000,
    })
};

const TILE_LAYOUT: Layout = unsafe { Layout::from_size_align_unchecked(8 * 8 / 2, 8 * 8 / 2) };

#[cfg(debug_assertions)]
unsafe fn debug_unreachable_unchecked(message: &'static str) -> ! {
    unreachable!("{}", message);
}

#[cfg(not(debug_assertions))]
const unsafe fn debug_unreachable_unchecked(_message: &'static str) -> ! {
    use core::hint::unreachable_unchecked;

    unreachable_unchecked();
}

#[derive(Clone, Copy, Debug)]
pub enum TileFormat {
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

pub struct TileSet<'a> {
    tiles: &'a [u32],
    format: TileFormat,
}

impl<'a> TileSet<'a> {
    pub fn new(tiles: &'a [u32], format: TileFormat) -> Self {
        Self { tiles, format }
    }

    fn num_tiles(&self) -> usize {
        self.tiles.len() / self.format.tile_size() * 4
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TileSetReference {
    id: u16,
    generation: u16,
}

impl TileSetReference {
    fn new(id: u16, generation: u16) -> Self {
        Self { id, generation }
    }
}

#[derive(Debug)]
pub struct TileIndex(u16);

impl TileIndex {
    pub(crate) const fn new(index: usize) -> Self {
        Self(index as u16)
    }

    pub(crate) const fn index(&self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct TileReference(NonNull<u32>);

enum ArenaStorageItem<T> {
    EndOfFreeList,
    NextFree(usize),
    Data(T, u16),
}

pub struct VRamManager<'a> {
    tilesets: Vec<ArenaStorageItem<TileSet<'a>>>,
    generation: u16,
    free_pointer: Option<usize>,

    tile_set_to_vram: Vec<Vec<Option<TileReference>>>,
    reference_counts: Vec<(u16, Option<(TileSetReference, u16)>)>,
}

impl<'a> VRamManager<'a> {
    pub fn new() -> Self {
        Self {
            tilesets: Vec::new(),
            generation: 0,
            free_pointer: None,

            tile_set_to_vram: Default::default(),
            reference_counts: Default::default(),
        }
    }

    pub fn add_tileset(&mut self, tileset: TileSet<'a>) -> TileSetReference {
        let generation = self.generation;
        self.generation = self.generation.wrapping_add(1);

        let num_tiles = tileset.num_tiles();
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
                _ => unsafe { debug_unreachable_unchecked("Free pointer cannot point to data") },
            }
        } else {
            self.tilesets.push(tileset);
            self.tilesets.len() - 1
        };

        self.tile_set_to_vram
            .resize(self.tilesets.len(), Default::default());
        self.tile_set_to_vram[index] = vec![Default::default(); num_tiles];

        TileSetReference::new(index as u16, generation)
    }

    pub fn remove_tileset(&mut self, tile_set_ref: TileSetReference) {
        let tileset = &self.tilesets[tile_set_ref.id as usize];

        match tileset {
            ArenaStorageItem::Data(_, generation) => {
                debug_assert_eq!(
                    *generation, tile_set_ref.generation,
                    "Tileset generation must be the same when removing"
                );

                self.tilesets[tile_set_ref.id as usize] = if let Some(ptr) = self.free_pointer {
                    ArenaStorageItem::NextFree(ptr)
                } else {
                    ArenaStorageItem::EndOfFreeList
                };

                self.free_pointer = Some(tile_set_ref.id as usize);
            }
            _ => panic!("Must remove valid tileset"),
        }
    }

    fn index_from_reference(reference: TileReference) -> usize {
        let difference = reference.0.as_ptr() as usize - TILE_RAM_START;
        difference / (8 * 8 / 2)
    }

    fn reference_from_index(index: TileIndex) -> TileReference {
        let ptr = (index.index() * (8 * 8 / 2)) as usize + TILE_RAM_START;
        TileReference(NonNull::new(ptr as *mut _).unwrap())
    }

    pub(crate) fn add_tile(&mut self, tile_set_ref: TileSetReference, tile: u16) -> TileIndex {
        let reference = self.tile_set_to_vram[tile_set_ref.id as usize][tile as usize];

        if let Some(reference) = reference {
            let index = Self::index_from_reference(reference);
            self.reference_counts[index].0 += 1;
            return TileIndex::new(index);
        }

        let new_reference: NonNull<u32> =
            unsafe { TILE_ALLOCATOR.alloc(TILE_LAYOUT) }.unwrap().cast();

        let tile_slice = if let ArenaStorageItem::Data(data, generation) =
            &self.tilesets[tile_set_ref.id as usize]
        {
            debug_assert_eq!(
                *generation, tile_set_ref.generation,
                "Stale tile data requested"
            );

            let tile_offset = (tile as usize) * data.format.tile_size() / 4;
            &data.tiles[tile_offset..(tile_offset + data.format.tile_size() / 4)]
        } else {
            panic!("Tile set ref must point to existing tile set");
        };

        let tile_size_in_half_words = TileFormat::FourBpp.tile_size() / 2;

        unsafe {
            dma_copy16(
                tile_slice.as_ptr() as *const u16,
                new_reference.as_ptr() as *mut u16,
                tile_size_in_half_words,
            );
        }

        let tile_reference = TileReference(new_reference);

        let index = Self::index_from_reference(tile_reference);

        self.tile_set_to_vram[tile_set_ref.id as usize][tile as usize] = Some(tile_reference);

        self.reference_counts
            .resize(self.reference_counts.len().max(index + 1), (0, None));

        self.reference_counts[index] = (1, Some((tile_set_ref, tile)));

        TileIndex::new(index)
    }

    pub(crate) fn remove_tile(&mut self, tile_index: TileIndex) {
        let index = tile_index.index() as usize;
        assert!(
            self.reference_counts[index].0 > 0,
            "Trying to decrease the reference count of {} below 0",
            index
        );

        self.reference_counts[index].0 -= 1;

        if self.reference_counts[index].0 != 0 {
            return;
        }

        let tile_reference = Self::reference_from_index(tile_index);
        unsafe {
            TILE_ALLOCATOR.dealloc(tile_reference.0.cast().as_ptr(), TILE_LAYOUT);
        }

        let tile_ref = self.reference_counts[index].1.unwrap();
        self.tile_set_to_vram[tile_ref.0.id as usize][tile_ref.1 as usize] = None;
        self.reference_counts[index].1 = None;
    }

    /// Copies raw palettes to the background palette without any checks.
    pub fn set_background_palette_raw(&mut self, palette: &[u16]) {
        unsafe {
            dma_copy16(palette.as_ptr(), PALETTE_BACKGROUND.as_ptr(), palette.len());
        }
    }

    fn set_background_palette(&mut self, pal_index: u8, palette: &palette16::Palette16) {
        unsafe {
            dma_copy16(
                palette.colours.as_ptr(),
                PALETTE_BACKGROUND.as_ptr().add(16 * pal_index as usize),
                palette.colours.len(),
            );
        }
    }

    /// Copies palettes to the background palettes without any checks.
    pub fn set_background_palettes(&mut self, palettes: &[palette16::Palette16]) {
        for (palette_index, entry) in palettes.iter().enumerate() {
            self.set_background_palette(palette_index as u8, entry)
        }
    }
}
