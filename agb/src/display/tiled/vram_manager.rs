use core::{alloc::Layout, ptr::NonNull};

use alloc::vec::Vec;

use crate::{
    agb_alloc::{block_allocator::BlockAllocator, bump_allocator::StartEnd},
    display::palette16,
    dma::dma_copy16,
    hash_map::HashMap,
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
    tiles: &'a [u8],
    format: TileFormat,
}

impl<'a> TileSet<'a> {
    pub fn new(tiles: &'a [u8], format: TileFormat) -> Self {
        Self { tiles, format }
    }

    fn reference(&self) -> NonNull<[u8]> {
        self.tiles.into()
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

#[derive(Clone, PartialEq, Eq, Hash)]
struct TileInTileSetReference {
    tileset: NonNull<[u8]>,
    tile: u16,
}

impl TileInTileSetReference {
    fn new(tileset: &'_ TileSet<'_>, tile: u16) -> Self {
        Self {
            tileset: tileset.reference(),
            tile,
        }
    }
}

#[derive(Clone, Default)]
struct TileReferenceCount {
    reference_count: u16,
    tile_in_tile_set: Option<TileInTileSetReference>,
}

impl TileReferenceCount {
    fn new(tile_in_tile_set: TileInTileSetReference) -> Self {
        Self {
            reference_count: 1,
            tile_in_tile_set: Some(tile_in_tile_set),
        }
    }

    fn increment_reference_count(&mut self) {
        self.reference_count += 1;
    }

    fn decrement_reference_count(&mut self) -> u16 {
        assert!(
            self.reference_count > 0,
            "Trying to decrease the reference count below 0",
        );

        self.reference_count -= 1;
        self.reference_count
    }

    fn clear(&mut self) {
        self.reference_count = 0;
        self.tile_in_tile_set = None;
    }
}

pub struct VRamManager {
    tile_set_to_vram: HashMap<TileInTileSetReference, TileReference>,
    reference_counts: Vec<TileReferenceCount>,
}

impl VRamManager {
    pub(crate) fn new() -> Self {
        let tile_set_to_vram: HashMap<TileInTileSetReference, TileReference> =
            HashMap::with_capacity(256);

        Self {
            tile_set_to_vram,
            reference_counts: Default::default(),
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

    pub(crate) fn add_tile(&mut self, tile_set: &TileSet<'_>, tile: u16) -> TileIndex {
        let reference = self
            .tile_set_to_vram
            .get(&TileInTileSetReference::new(tile_set, tile));

        if let Some(reference) = reference {
            let index = Self::index_from_reference(*reference);
            self.reference_counts[index].increment_reference_count();
            return TileIndex::new(index);
        }

        let new_reference: NonNull<u32> =
            unsafe { TILE_ALLOCATOR.alloc(TILE_LAYOUT) }.unwrap().cast();
        let tile_reference = TileReference(new_reference);

        self.copy_tile_to_location(tile_set, tile, tile_reference);

        let index = Self::index_from_reference(tile_reference);

        self.tile_set_to_vram
            .insert(TileInTileSetReference::new(tile_set, tile), tile_reference);

        self.reference_counts.resize(
            self.reference_counts.len().max(index + 1),
            Default::default(),
        );

        self.reference_counts[index] =
            TileReferenceCount::new(TileInTileSetReference::new(tile_set, tile));

        TileIndex::new(index)
    }

    pub(crate) fn remove_tile(&mut self, tile_index: TileIndex) {
        let index = tile_index.index() as usize;

        let new_reference_count = self.reference_counts[index].decrement_reference_count();

        if new_reference_count != 0 {
            return;
        }

        let tile_reference = Self::reference_from_index(tile_index);
        unsafe {
            TILE_ALLOCATOR.dealloc_no_normalise(tile_reference.0.cast().as_ptr(), TILE_LAYOUT);
        }

        let tile_ref = self.reference_counts[index]
            .tile_in_tile_set
            .as_ref()
            .unwrap();

        self.tile_set_to_vram.remove(tile_ref);
        self.reference_counts[index].clear();
    }

    pub fn replace_tile(
        &mut self,
        source_tile_set: &TileSet<'_>,
        source_tile: u16,
        target_tile_set: &TileSet<'_>,
        target_tile: u16,
    ) {
        if let Some(&reference) = self
            .tile_set_to_vram
            .get(&TileInTileSetReference::new(source_tile_set, source_tile))
        {
            self.copy_tile_to_location(target_tile_set, target_tile, reference);
        }
    }

    fn copy_tile_to_location(
        &self,
        tile_set: &TileSet<'_>,
        tile_id: u16,
        tile_reference: TileReference,
    ) {
        let tile_size = tile_set.format.tile_size();
        let tile_offset = (tile_id as usize) * tile_size;
        let tile_slice = &tile_set.tiles[tile_offset..(tile_offset + tile_size)];

        let tile_size_in_half_words = tile_slice.len() / 2;

        let target_location = tile_reference.0.as_ptr() as *mut _;

        unsafe {
            dma_copy16(
                tile_slice.as_ptr() as *const u16,
                target_location,
                tile_size_in_half_words,
            )
        };
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
