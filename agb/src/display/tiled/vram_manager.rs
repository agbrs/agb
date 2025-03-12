use core::{alloc::Layout, mem::MaybeUninit, ptr::NonNull};

use alloc::{slice, vec::Vec};

use crate::{
    agb_alloc::{block_allocator::BlockAllocator, bump_allocator::StartEnd},
    display::palette16,
    dma,
    hash_map::{Entry, HashMap},
    memory_mapped::MemoryMapped1DArray,
    util::SyncUnsafeCell,
};

use super::{CHARBLOCK_SIZE, VRAM_START};

const PALETTE_BACKGROUND: MemoryMapped1DArray<u16, 256> =
    unsafe { MemoryMapped1DArray::new(0x0500_0000) };

static TILE_ALLOCATOR: BlockAllocator = unsafe {
    BlockAllocator::new(StartEnd {
        start: || VRAM_START + 8 * 8,
        end: || VRAM_START + CHARBLOCK_SIZE * 2,
    })
};

const fn layout_of(format: TileFormat) -> Layout {
    unsafe { Layout::from_size_align_unchecked(format.tile_size(), format.tile_size()) }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileFormat {
    FourBpp = 5,
    EightBpp = 6,
}

impl TileFormat {
    /// Returns the size of the tile in bytes
    pub(crate) const fn tile_size(self) -> usize {
        1 << self as usize
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
        debug_assert!(self.tile_in_tile_set.is_some());

        self.reference_count += 1;
    }

    fn decrement_reference_count(&mut self) -> u16 {
        assert!(
            self.reference_count > 0,
            "Trying to decrease the reference count below 0",
        );

        debug_assert!(self.tile_in_tile_set.is_some());

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
pub struct DynamicTile {
    pub tile_data: &'static mut [u32],
}

impl DynamicTile {
    #[must_use]
    pub fn new() -> Self {
        VRAM_MANAGER.new_dynamic_tile()
    }

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

    #[must_use]
    pub(crate) fn tile_set(&self) -> TileSet<'_> {
        let tiles = unsafe {
            slice::from_raw_parts_mut(
                VRAM_START as *mut u8,
                1024 * TileFormat::FourBpp.tile_size(),
            )
        };

        TileSet::new(tiles, TileFormat::FourBpp)
    }

    #[must_use]
    pub(crate) fn tile_id(&self) -> u16 {
        let difference = self.tile_data.as_ptr() as usize - VRAM_START;
        (difference / TileFormat::FourBpp.tile_size()) as u16
    }
}

impl Default for DynamicTile {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for DynamicTile {
    fn drop(&mut self) {
        unsafe {
            VRAM_MANAGER.drop_dynamic_tile(self);
        }
    }
}

pub struct VRamManager {
    inner: SyncUnsafeCell<MaybeUninit<VRamManagerInner>>,
}

// SAFETY: This is the _only_ one
pub static VRAM_MANAGER: VRamManager = unsafe { VRamManager::new() };

impl VRamManager {
    const unsafe fn new() -> Self {
        Self {
            inner: SyncUnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    // Should only be called in the Gba struct's constructor to ensure it happens exactly once
    pub(crate) unsafe fn initialise(&self) {
        unsafe {
            (*self.inner.get()).write(VRamManagerInner::new());
        }
    }

    fn with<T>(&self, f: impl FnOnce(&mut VRamManagerInner) -> T) -> T {
        f(unsafe { (*self.inner.get()).assume_init_mut() })
    }
}

impl VRamManager {
    unsafe fn drop_dynamic_tile(&self, tile: &DynamicTile) {
        self.with(|inner| inner.remove_dynamic_tile(tile));
    }

    pub(crate) fn new_dynamic_tile(&self) -> DynamicTile {
        self.with(VRamManagerInner::new_dynamic_tile)
    }

    pub(crate) fn remove_tile(&self, index: TileIndex) {
        self.with(|inner| inner.remove_tile(index));
    }

    pub(crate) fn increase_reference(&self, index: TileIndex) {
        self.with(|inner| inner.increase_reference(index));
    }

    pub(crate) fn add_tile(&self, tile_set: &TileSet<'_>, tile_index: u16) -> TileIndex {
        self.with(|inner| inner.add_tile(tile_set, tile_index))
    }

    pub(crate) fn gc(&self) {
        self.with(VRamManagerInner::gc);
    }

    pub fn set_background_palette(&self, pal_index: u8, palette: &palette16::Palette16) {
        self.with(|inner| inner.set_background_palette(pal_index, palette));
    }

    pub fn set_background_palettes(&self, palettes: &[palette16::Palette16]) {
        self.with(|inner| inner.set_background_palettes(palettes));
    }

    pub fn replace_tile(
        &self,
        source_tile_set: &TileSet<'_>,
        source_tile: u16,
        target_tile_set: &TileSet<'_>,
        target_tile: u16,
    ) {
        self.with(|inner| {
            inner.replace_tile(source_tile_set, source_tile, target_tile_set, target_tile);
        });
    }

    #[must_use]
    pub fn background_palette_colour_dma(
        &self,
        pal_index: usize,
        colour_index: usize,
    ) -> dma::DmaControllable<u16> {
        self.with(|inner| inner.background_palette_colour_dma(pal_index, colour_index))
    }

    pub fn set_background_palette_colour(
        &mut self,
        pal_index: usize,
        colour_index: usize,
        colour: u16,
    ) {
        self.with(|inner| inner.set_background_palette_colour(pal_index, colour_index, colour));
    }

    /// Gets the index of the colour for a given background palette, or None if it doesn't exist
    #[must_use]
    pub fn find_colour_index_16(&self, palette_index: usize, colour: u16) -> Option<usize> {
        self.with(|inner| inner.find_colour_index_16(palette_index, colour))
    }

    /// Gets the index of the colour in the entire background palette, or None if it doesn't exist
    #[must_use]
    pub fn find_colour_index_256(&self, colour: u16) -> Option<usize> {
        self.with(|inner| inner.find_colour_index_256(colour))
    }
}

struct VRamManagerInner {
    tile_set_to_vram: HashMap<TileInTileSetReference, TileReference>,
    reference_counts: Vec<TileReferenceCount>,

    indices_to_gc: Vec<TileIndex>,
}

impl VRamManagerInner {
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
        let difference = reference.0.as_ptr() as usize - VRAM_START;
        TileIndex::new(difference / format.tile_size(), format)
    }

    fn reference_from_index(index: TileIndex) -> TileReference {
        let ptr = (index.raw_index() as usize * index.format().tile_size()) + VRAM_START;
        TileReference(NonNull::new(ptr as *mut _).unwrap())
    }

    #[must_use]
    pub fn new_dynamic_tile(&mut self) -> DynamicTile {
        // TODO: format param?
        let tile_format = TileFormat::FourBpp;
        let new_reference: NonNull<u32> = unsafe { TILE_ALLOCATOR.alloc(layout_of(tile_format)) }
            .unwrap()
            .cast();
        let tile_reference = TileReference(new_reference);

        let index = Self::index_from_reference(tile_reference, tile_format);
        let key = index.refcount_key();

        let tiles = unsafe {
            slice::from_raw_parts_mut(VRAM_START as *mut u8, 1024 * tile_format.tile_size())
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
    fn remove_dynamic_tile(&mut self, dynamic_tile: &DynamicTile) {
        let pointer = NonNull::new(dynamic_tile.tile_data.as_ptr() as *mut _).unwrap();
        let tile_reference = TileReference(pointer);

        // TODO: dynamic_tile.format?
        let tile_index = Self::index_from_reference(tile_reference, TileFormat::FourBpp);
        self.remove_tile(tile_index);
    }

    #[inline(never)]
    pub(crate) fn add_tile(&mut self, tile_set: &TileSet<'_>, tile: u16) -> TileIndex {
        let reference = self
            .tile_set_to_vram
            .entry(TileInTileSetReference::new(tile_set, tile));

        if let Entry::Occupied(reference) = reference {
            let tile_index = Self::index_from_reference(*reference.get(), tile_set.format);
            self.increase_reference(tile_index);

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

    fn increase_reference(&mut self, tile_index: TileIndex) {
        let key = tile_index.refcount_key();
        self.reference_counts[key].increment_reference_count();
    }

    pub(crate) fn gc(&mut self) {
        for tile_index in self.indices_to_gc.drain(..) {
            let key = tile_index.refcount_key();
            if self.reference_counts[key].current_count() > 0 {
                continue; // it has since been added back
            }

            let Some(tile_ref) = self.reference_counts[key].tile_in_tile_set.as_ref() else {
                // already been deleted
                continue;
            };

            let tile_reference = Self::reference_from_index(tile_index);
            unsafe {
                TILE_ALLOCATOR.dealloc(
                    tile_reference.0.cast().as_ptr(),
                    layout_of(tile_index.format()),
                );
            }

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

    /// Copies the palette to the given palette index
    pub fn set_background_palette(&mut self, pal_index: u8, palette: &palette16::Palette16) {
        assert!(pal_index < 16);
        for (colour_index, &colour) in palette.colours.iter().enumerate() {
            PALETTE_BACKGROUND.set(colour_index + 16 * pal_index as usize, colour);
        }
    }

    /// The DMA register for controlling a single colour in a single background. Good for drawing gradients
    #[must_use]
    pub fn background_palette_colour_dma(
        &self,
        pal_index: usize,
        colour_index: usize,
    ) -> dma::DmaControllable<u16> {
        assert!(pal_index < 16);
        assert!(colour_index < 16);

        unsafe {
            dma::DmaControllable::new(
                PALETTE_BACKGROUND
                    .as_ptr()
                    .add(16 * pal_index + colour_index),
            )
        }
    }

    /// Sets a single colour for a given background palette. Takes effect immediately
    pub fn set_background_palette_colour(
        &mut self,
        pal_index: usize,
        colour_index: usize,
        colour: u16,
    ) {
        assert!(pal_index < 16);
        assert!(colour_index < 16);

        PALETTE_BACKGROUND.set(colour_index + 16 * pal_index, colour);
    }

    /// Copies palettes to the background palettes without any checks.
    pub fn set_background_palettes(&mut self, palettes: &[palette16::Palette16]) {
        for (palette_index, entry) in palettes.iter().enumerate() {
            self.set_background_palette(palette_index as u8, entry);
        }
    }

    /// Gets the index of the colour for a given background palette, or None if it doesn't exist
    #[must_use]
    pub fn find_colour_index_16(&self, palette_index: usize, colour: u16) -> Option<usize> {
        assert!(palette_index < 16);

        (0..16).find(|i| PALETTE_BACKGROUND.get(palette_index * 16 + i) == colour)
    }

    /// Gets the index of the colour in the entire background palette, or None if it doesn't exist
    #[must_use]
    pub fn find_colour_index_256(&self, colour: u16) -> Option<usize> {
        (0..256).find(|&i| PALETTE_BACKGROUND.get(i) == colour)
    }
}
