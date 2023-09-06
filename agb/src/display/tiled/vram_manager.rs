use core::{alloc::Layout, ptr::NonNull};

use alloc::{slice, vec::Vec};

use crate::{
    agb_alloc::{block_allocator::BlockAllocator, bump_allocator::StartEnd},
    display::palette16,
    dma::dma_copy16,
    hash_map::{Entry, HashMap},
    memory_mapped::MemoryMapped1DArray,
};

use super::TileSetting;

const TILE_RAM_START: usize = 0x0600_0000;

const PALETTE_BACKGROUND: MemoryMapped1DArray<u16, 256> =
    unsafe { MemoryMapped1DArray::new(0x0500_0000) };

static TILE_ALLOCATOR: BlockAllocator = unsafe {
    BlockAllocator::new(StartEnd {
        start: || TILE_RAM_START + 8 * 8,
        end: || TILE_RAM_START + 0x8000,
    })
};

const fn layout_of(format: TileFormat) -> Layout {
    unsafe { Layout::from_size_align_unchecked(format.tile_size(), format.tile_size()) }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub enum TileFormat {
    FourBpp = 0,
    EightBpp = 1,
}

impl TileFormat {
    /// Returns the size of the tile in bytes
    pub(crate) const fn tile_size(self) -> usize {
        match self {
            TileFormat::FourBpp => 8 * 8 / 2,
            TileFormat::EightBpp => 8 * 8,
        }
    }
}

pub struct TileSet<'a> {
    tiles: &'a [u8],
    format: TileFormat,
}

impl<'a> TileSet<'a> {
    #[must_use]
    pub const fn new(tiles: &'a [u8], format: TileFormat) -> Self {
        Self { tiles, format }
    }

    #[must_use]
    pub const fn format(&self) -> TileFormat {
        self.format
    }

    fn reference(&self) -> NonNull<[u8]> {
        self.tiles.into()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TileIndex {
    FourBpp(u16),
    EightBpp(u16),
}

impl TileIndex {
    pub(crate) const fn new(index: usize, format: TileFormat) -> Self {
        match format {
            TileFormat::FourBpp => Self::FourBpp(index as u16),
            TileFormat::EightBpp => Self::EightBpp(index as u16),
        }
    }

    pub(crate) const fn raw_index(self) -> u16 {
        match self {
            TileIndex::FourBpp(x) => x,
            TileIndex::EightBpp(x) => x,
        }
    }

    pub(crate) const fn format(self) -> TileFormat {
        match self {
            TileIndex::FourBpp(_) => TileFormat::FourBpp,
            TileIndex::EightBpp(_) => TileFormat::EightBpp,
        }
    }

    fn refcount_key(self) -> usize {
        match self {
            TileIndex::FourBpp(x) => x as usize,
            TileIndex::EightBpp(x) => x as usize * 2,
        }
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

    fn current_count(&self) -> u16 {
        self.reference_count
    }
}

#[non_exhaustive]
pub struct DynamicTile<'a> {
    pub tile_data: &'a mut [u32],
}

impl DynamicTile<'_> {
    #[must_use]
    pub fn fill_with(self, colour_index: u8) -> Self {
        let colour_index = u32::from(colour_index);

        let mut value = 0;
        for i in 0..8 {
            value |= colour_index << (i * 4);
        }

        self.tile_data.fill(value);
        self
    }
}

impl DynamicTile<'_> {
    #[must_use]
    pub fn tile_set(&self) -> TileSet<'_> {
        let tiles = unsafe {
            slice::from_raw_parts_mut(
                TILE_RAM_START as *mut u8,
                1024 * TileFormat::FourBpp.tile_size(),
            )
        };

        TileSet::new(tiles, TileFormat::FourBpp)
    }

    #[must_use]
    pub fn tile_setting(&self) -> TileSetting {
        let difference = self.tile_data.as_ptr() as usize - TILE_RAM_START;
        let tile_id = (difference / TileFormat::FourBpp.tile_size()) as u16;

        TileSetting::new(tile_id, false, false, 0)
    }
}

pub struct VRamManager {
    tile_set_to_vram: HashMap<TileInTileSetReference, TileReference>,
    reference_counts: Vec<TileReferenceCount>,

    indices_to_gc: Vec<TileIndex>,
}

impl VRamManager {
    pub(crate) fn new() -> Self {
        let tile_set_to_vram: HashMap<TileInTileSetReference, TileReference> =
            HashMap::with_capacity(256);

        Self {
            tile_set_to_vram,
            reference_counts: Default::default(),
            indices_to_gc: Default::default(),
        }
    }

    fn index_from_reference(reference: TileReference, format: TileFormat) -> TileIndex {
        let difference = reference.0.as_ptr() as usize - TILE_RAM_START;
        TileIndex::new(difference / format.tile_size(), format)
    }

    fn reference_from_index(index: TileIndex) -> TileReference {
        let ptr = (index.raw_index() as usize * index.format().tile_size()) + TILE_RAM_START;
        TileReference(NonNull::new(ptr as *mut _).unwrap())
    }

    #[must_use]
    pub fn new_dynamic_tile<'a>(&mut self) -> DynamicTile<'a> {
        // TODO: format param?
        let tile_format = TileFormat::FourBpp;
        let new_reference: NonNull<u32> = unsafe { TILE_ALLOCATOR.alloc(layout_of(tile_format)) }
            .unwrap()
            .cast();
        let tile_reference = TileReference(new_reference);

        let index = Self::index_from_reference(tile_reference, tile_format);
        let key = index.refcount_key();

        let tiles = unsafe {
            slice::from_raw_parts_mut(TILE_RAM_START as *mut u8, 1024 * tile_format.tile_size())
        };

        let tile_set = TileSet::new(tiles, tile_format);

        self.tile_set_to_vram.insert(
            TileInTileSetReference::new(&tile_set, index.raw_index()),
            tile_reference,
        );

        self.reference_counts
            .resize(self.reference_counts.len().max(key + 1), Default::default());
        self.reference_counts[key] =
            TileReferenceCount::new(TileInTileSetReference::new(&tile_set, index.raw_index()));

        DynamicTile {
            tile_data: unsafe {
                slice::from_raw_parts_mut(
                    tiles
                        .as_mut_ptr()
                        .add(index.raw_index() as usize * tile_format.tile_size())
                        .cast(),
                    tile_format.tile_size() / core::mem::size_of::<u32>(),
                )
            },
        }
    }

    // This needs to take ownership of the dynamic tile because it will no longer be valid after this call
    #[allow(clippy::needless_pass_by_value)]
    pub fn remove_dynamic_tile(&mut self, dynamic_tile: DynamicTile<'_>) {
        let pointer = NonNull::new(dynamic_tile.tile_data.as_mut_ptr() as *mut _).unwrap();
        let tile_reference = TileReference(pointer);

        // TODO: dynamic_tile.format?
        let tile_index = Self::index_from_reference(tile_reference, TileFormat::FourBpp);
        self.remove_tile(tile_index);
    }

    pub(crate) fn add_tile(&mut self, tile_set: &TileSet<'_>, tile: u16) -> TileIndex {
        let reference = self
            .tile_set_to_vram
            .entry(TileInTileSetReference::new(tile_set, tile));

        if let Entry::Occupied(reference) = reference {
            let tile_index = Self::index_from_reference(*reference.get(), tile_set.format);
            let key = tile_index.refcount_key();
            self.reference_counts[key].increment_reference_count();
            return tile_index;
        }

        let new_reference: NonNull<u32> =
            unsafe { TILE_ALLOCATOR.alloc(layout_of(tile_set.format)) }
                .expect("Ran out of video RAM for tiles")
                .cast();
        let tile_reference = TileReference(new_reference);
        reference.or_insert(tile_reference);

        self.copy_tile_to_location(tile_set, tile, tile_reference);

        let index = Self::index_from_reference(tile_reference, tile_set.format);
        let key = index.refcount_key();

        self.reference_counts
            .resize(self.reference_counts.len().max(key + 1), Default::default());

        self.reference_counts[key] =
            TileReferenceCount::new(TileInTileSetReference::new(tile_set, tile));

        index
    }

    pub(crate) fn remove_tile(&mut self, tile_index: TileIndex) {
        let key = tile_index.refcount_key();

        let new_reference_count = self.reference_counts[key].decrement_reference_count();

        if new_reference_count != 0 {
            return;
        }

        self.indices_to_gc.push(tile_index);
    }

    pub(crate) fn gc(&mut self) {
        for tile_index in self.indices_to_gc.drain(..) {
            let key = tile_index.refcount_key();
            if self.reference_counts[key].current_count() > 0 {
                continue; // it has since been added back
            }

            let tile_reference = Self::reference_from_index(tile_index);
            unsafe {
                TILE_ALLOCATOR.dealloc(
                    tile_reference.0.cast().as_ptr(),
                    layout_of(tile_index.format()),
                );
            }

            let tile_ref = self.reference_counts[key]
                .tile_in_tile_set
                .as_ref()
                .unwrap();

            self.tile_set_to_vram.remove(tile_ref);
            self.reference_counts[key].clear();
        }
    }

    pub fn replace_tile(
        &mut self,
        source_tile_set: &TileSet<'_>,
        source_tile: u16,
        target_tile_set: &TileSet<'_>,
        target_tile: u16,
    ) {
        assert_eq!(
            source_tile_set.format, target_tile_set.format,
            "Must replace a tileset with the same format"
        );

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
        let tile_format = tile_set.format;
        let tile_size = tile_format.tile_size();
        let tile_offset = (tile_id as usize) * tile_size;
        let tile_data_start = unsafe { tile_set.tiles.as_ptr().add(tile_offset) };

        let target_location = tile_reference.0.as_ptr() as *mut _;

        unsafe {
            match tile_format {
                TileFormat::FourBpp => core::arch::asm!(
                    ".rept 2",
                    "ldmia {src}!, {{{tmp1},{tmp2},{tmp3},{tmp4}}}",
                    "stmia {dest}!, {{{tmp1},{tmp2},{tmp3},{tmp4}}}",
                    ".endr",
                    src = inout(reg) tile_data_start => _,
                    dest = inout(reg) target_location => _,
                    tmp1 = out(reg) _,
                    tmp2 = out(reg) _,
                    tmp3 = out(reg) _,
                    tmp4 = out(reg) _,
                ),
                TileFormat::EightBpp => core::arch::asm!(
                    ".rept 4",
                    "ldmia {src}!, {{{tmp1},{tmp2},{tmp3},{tmp4}}}",
                    "stmia {dest}!, {{{tmp1},{tmp2},{tmp3},{tmp4}}}",
                    ".endr",
                    src = inout(reg) tile_data_start => _,
                    dest = inout(reg) target_location => _,
                    tmp1 = out(reg) _,
                    tmp2 = out(reg) _,
                    tmp3 = out(reg) _,
                    tmp4 = out(reg) _,
                ),
            }
        }
    }

    /// Copies raw palettes to the background palette without any checks.
    pub fn set_background_palette_raw(&mut self, palette: &[u16]) {
        unsafe {
            dma_copy16(palette.as_ptr(), PALETTE_BACKGROUND.as_ptr(), palette.len());
        }
    }

    fn set_background_palette(&mut self, pal_index: u8, palette: &palette16::Palette16) {
        for (colour_index, &colour) in palette.colours.iter().enumerate() {
            PALETTE_BACKGROUND.set(colour_index + 16 * pal_index as usize, colour);
        }
    }

    /// Copies palettes to the background palettes without any checks.
    pub fn set_background_palettes(&mut self, palettes: &[palette16::Palette16]) {
        for (palette_index, entry) in palettes.iter().enumerate() {
            self.set_background_palette(palette_index as u8, entry);
        }
    }
}
