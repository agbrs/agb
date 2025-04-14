#![warn(missing_docs)]
use core::{alloc::Layout, fmt::Debug, mem::MaybeUninit, ptr::NonNull};

use alloc::{slice, vec::Vec};

use crate::{
    agb_alloc::{block_allocator::BlockAllocator, bump_allocator::StartEnd},
    display::{Palette16, Rgb15},
    dma,
    hash_map::{Entry, HashMap},
    memory_mapped::MemoryMapped1DArray,
    util::SyncUnsafeCell,
};

use super::{CHARBLOCK_SIZE, VRAM_START};

const PALETTE_BACKGROUND: MemoryMapped1DArray<Rgb15, 256> =
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

/// Represents the pixel format of a tile in VRAM.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileFormat {
    /// 4 bits per pixel, allowing for 16 colours per tile
    FourBpp = 5,
    /// 8 bits per pixel, allowing for 256 colours per tile
    EightBpp = 6,
}

impl TileFormat {
    /// Returns the size of the tile in bytes
    pub(crate) const fn tile_size(self) -> usize {
        1 << self as usize
    }
}

/// Represents a collection of tile data in a specific format.
///
/// A `TileSet` holds a slice of raw byte data representing one or more tiles and the
/// format of those tiles (either 4 bits per pixel or 8 bits per pixel).
pub struct TileSet<'a> {
    tiles: &'a [u8],
    format: TileFormat,
}

impl<'a> TileSet<'a> {
    /// Create a new TileSet. You probably shouldn't use this function and instead rely on
    /// [`include_background_gfx!`](crate::include_background_gfx).
    #[must_use]
    pub const fn new(tiles: &'a [u8], format: TileFormat) -> Self {
        Self { tiles, format }
    }

    /// Returns the format used for this TileSet. This will be either [`TileFormat::FourBpp`] if
    /// it was imported as a 16 colour background (the default) or [`TileFormat::EightBpp`] if
    /// it was imported as a 256 colour background.
    #[must_use]
    pub const fn format(&self) -> TileFormat {
        self.format
    }

    fn reference(&self) -> NonNull<[u8]> {
        self.tiles.into()
    }
}

/// Represents the index of a tile within VRAM, along with its pixel format.
///
/// Tile indices are used to reference specific 8x8 pixel blocks of data stored in the GBA's
/// Video RAM (VRAM).
#[derive(Debug, Clone, Copy)]
pub(crate) enum TileIndex {
    /// 4 bits per pixel, allowing for 16 colours per tile
    FourBpp(u16),
    /// 8 bits per pixel, allowing for 256 colours per tile
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

/// Represents a tile that can be modified at runtime. Most tiles fetched using [`TileSet`] are generated at
/// compile time and are loaded on demand from ROM. Note that `DynamicTile16`s are always 16 colours, so four bits
/// per pixel are used.
///
/// If you have access to a `DynamicTile16`, then this is actually a direct pointer to Video RAM. Note that any
/// writes to [`.data()`](Self::data) must be at least 16-bits at a time, or it won't work due to how the GBA's video RAM
/// works.
///
/// While a DynamicTile16 is active, some of Video RAM will be used up by it, so ensure it is dropped when you don't
/// need it any more.
///
/// Most of the time, you won't need this. But it is used heavily in the
/// [`RegularBackgroundTextRenderer`](crate::display::font::RegularBackgroundTextRenderer).
#[non_exhaustive]
pub struct DynamicTile16 {
    /// The actual tile data. This will be exactly 8 long, where each entry represents one row of pixel data.
    tile_data: &'static mut [u32],
}

impl Debug for DynamicTile16 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::write!(f, "DynamicTile16({})", self.tile_id())
    }
}

impl DynamicTile16 {
    /// Creates a new `DynamicTile16`. Dynamic tiles aren't cleared by default, so the value you get in `tile_data`
    /// won't necessarily be empty, and will contain whatever was in that same location last time.
    ///
    /// If you are completely filling the tile yourself, then this doesn't matter, but otherwise you may want to
    /// do something like:
    ///
    /// ```rust,no_run
    /// # #![no_std]
    /// # #![no_main]
    /// use agb::display::tiled::DynamicTile16;
    ///
    /// # fn test() {
    /// let my_new_tile = DynamicTile16::new().fill_with(0);
    /// # }
    /// ```
    ///
    /// which will fill the tile with the transparent colour.
    #[must_use]
    pub fn new() -> Self {
        VRAM_MANAGER.new_dynamic_tile()
    }

    /// Fills a `DynamicTile16` with a given colour index from the palette. Note that the actual palette
    /// doesn't get assigned until you try to render it.
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

    /// Returns a reference to the underlying tile data. Note that you cannot write to this in 8-bit chunks
    /// and must write to it in at least 16-bit chunks.
    pub fn data(&mut self) -> &mut [u32] {
        self.tile_data
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

    /// Sets the pixel at `(x, y)` to the colour index given by `palette_index`
    pub fn set_pixel(&mut self, x: usize, y: usize, palette_index: u8) {
        assert!((0..9).contains(&x));
        assert!((0..9).contains(&y));
        assert!(palette_index < 16);

        let index = x + y * 8;
        // each 'pixel' is one nibble, so 8 nibbles in a word (u32)
        let word_index = index / 8;
        let nibble_offset = index % 8;

        let current_value = &mut self.tile_data[word_index];

        let mask = 0xf << (nibble_offset * 4);
        let palette_value = u32::from(palette_index) << (nibble_offset * 4);

        *current_value = (*current_value & !mask) | palette_value;
    }
}

impl Default for DynamicTile16 {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for DynamicTile16 {
    fn drop(&mut self) {
        unsafe {
            VRAM_MANAGER.drop_dynamic_tile(self);
        }
    }
}

/// Manages the allocation and deallocation of tile data within the Game Boy Advance's **Video RAM (VRAM)**.
///
/// VRAM is a **limited resource (96 KB)** on the GBA, and this manager helps to use it **efficiently**
/// by tracking the usage of tiles (8x8 pixel mini-bitmaps) and allowing for **dynamic allocation**
/// of tile memory.
///
/// The `VRamManager` interacts with several key GBA graphics concepts:
///
/// *   **Tiles:** The fundamental building blocks of graphics on the GBA. This manager tracks which
///     tiles are in use and where they are located in VRAM.
/// *   **Palettes:** Colour palettes stored in PAL RAM (both background and sprite palettes). The
///     `VRamManager` provides methods for interacting with the background palette.
///
/// To ensure efficient memory usage and prevent premature freeing of shared resources, the
/// `VRamManager` employs **reference counting** for tiles that are used by multiple parts of the
/// game (e.g., different sprites or background layers).
///
/// Additionally, the manager supports **dynamic allocation** of tiles for temporary needs, allowing
/// for flexible memory management during gameplay. These dynamically allocated tiles have their
/// memory managed by the `VRamManager` and are freed when no longer in use.
///
/// All interactions for the VRamManager is done via the static [`VRAM_MANAGER`] instance.
pub struct VRamManager {
    inner: SyncUnsafeCell<MaybeUninit<VRamManagerInner>>,
}

/// The global instance of the VRamManager. You should always use this and never attempt to
/// construct one yourself.
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
    unsafe fn drop_dynamic_tile(&self, tile: &DynamicTile16) {
        self.with(|inner| unsafe { inner.remove_dynamic_tile(tile) });
    }

    pub(crate) fn new_dynamic_tile(&self) -> DynamicTile16 {
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

    /// Sets the `pal_index` background palette to the 4bpp one given in `palette`.
    /// Note that `pal_index` must be in the range 0..=15 as there are only 16 palettes available on
    /// the GameBoy Advance.
    pub fn set_background_palette(&self, pal_index: u8, palette: &Palette16) {
        self.with(|inner| inner.set_background_palette(pal_index, palette));
    }

    /// Sets all background palettes based on the entries given in `palettes`. Note that the GameBoy Advance
    /// can have at most 16 palettes loaded at once, so only the first 16 will be loaded (although this
    /// array can be shorter if you don't need all 16).
    ///
    /// You will probably call this method early on in the game setup using the palette combination that you
    /// built using [`include_background_gfx!`](crate::include_background_gfx).
    pub fn set_background_palettes(&self, palettes: &[Palette16]) {
        self.with(|inner| inner.set_background_palettes(palettes));
    }

    /// Replaces all instances of the tile found in the `source_tile_set` `source_tile` combination with
    /// the one in `target_tile_set` `target_tile`. This will just do nothing if don't have any occurrences
    /// of the `source_tile_set` `source_tile` combination.
    ///
    /// This is primarily intended for use with animated backgrounds since it is incredibly efficient, only
    /// modifying the tile data once.
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

    /// Used if you want to control a colour in the background which could change e.g. on every row of pixels.
    /// Very useful if you want a gradient of more colours than the gba can normally handle.
    ///
    /// See [`HBlankDmaDefinition`](crate::dma::HBlankDmaDefinition) for examples for how to do this, or the
    /// [`dma_effect_background_colour`](https://agbrs.dev/examples/dma_effect_background_colour) example.
    #[must_use]
    pub fn background_palette_colour_dma(
        &self,
        pal_index: usize,
        colour_index: usize,
    ) -> dma::DmaControllable<Rgb15> {
        self.with(|inner| inner.background_palette_colour_dma(pal_index, colour_index))
    }

    /// Used if you want to control a colour in the background which could change e.g. on every row of pixels.
    /// Very useful if you want a gradient of more colours than the gba can normally handle.
    ///
    /// See [`HBlankDmaDefinition`](crate::dma::HBlankDmaDefinition) for examples for how to do this, or the
    /// [`dma_effect_background_colour`](https://agbrs.dev/examples/dma_effect_background_colour) example.
    #[must_use]
    pub fn background_palette_colour_256_dma(
        &self,
        colour_index: usize,
    ) -> dma::DmaControllable<Rgb15> {
        assert!(colour_index < 256);

        self.background_palette_colour_dma(colour_index / 16, colour_index % 16)
    }

    /// Set a single colour in a single palette. `pal_index` must be in 0..16 as must colour_index.
    /// If you're working with a 256 colour palette, you should use [`VRamManager::set_background_palette_colour_256()`]
    /// instead. Although these use the same underlying palette, so both methods will work.
    pub fn set_background_palette_colour(
        &self,
        pal_index: usize,
        colour_index: usize,
        colour: Rgb15,
    ) {
        self.with(|inner| inner.set_background_palette_colour(pal_index, colour_index, colour));
    }

    /// Sets a single colour in a 256 colour palette. `colour_index` must be less than 256.
    pub fn set_background_palette_colour_256(&self, colour_index: usize, colour: Rgb15) {
        assert!(colour_index < 256);
        self.set_background_palette_colour(colour_index / 16, colour_index % 16, colour);
    }

    /// Gets the index of the colour for a given background palette, or None if it doesn't exist
    #[must_use]
    pub fn find_colour_index_16(&self, palette_index: usize, colour: Rgb15) -> Option<usize> {
        self.with(|inner| inner.find_colour_index_16(palette_index, colour))
    }

    /// Gets the index of the colour in the entire background palette, or None if it doesn't exist
    #[must_use]
    pub fn find_colour_index_256(&self, colour: Rgb15) -> Option<usize> {
        self.with(|inner| inner.find_colour_index_256(colour))
    }
}

struct VRamManagerInner {
    tile_set_to_vram: HashMap<TileInTileSetReference, TileReference>,
    reference_counts: Vec<TileReferenceCount>,

    indices_to_gc: Vec<TileIndex>,
}

impl VRamManagerInner {
    fn new() -> Self {
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
    fn new_dynamic_tile(&mut self) -> DynamicTile16 {
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

        DynamicTile16 {
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

    // The dynamic tile because it will no longer be valid after this call
    unsafe fn remove_dynamic_tile(&mut self, dynamic_tile: &DynamicTile16) {
        let pointer = NonNull::new(dynamic_tile.tile_data.as_ptr() as *mut _).unwrap();
        let tile_reference = TileReference(pointer);

        // TODO: dynamic_tile.format?
        let tile_index = Self::index_from_reference(tile_reference, TileFormat::FourBpp);
        self.remove_tile(tile_index);
    }

    #[inline(never)]
    fn add_tile(&mut self, tile_set: &TileSet<'_>, tile: u16) -> TileIndex {
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

    fn remove_tile(&mut self, tile_index: TileIndex) {
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

    fn gc(&mut self) {
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

    fn replace_tile(
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
    fn set_background_palette(&mut self, pal_index: u8, palette: &Palette16) {
        assert!(pal_index < 16);
        for (colour_index, &colour) in palette.colours.iter().enumerate() {
            PALETTE_BACKGROUND.set(colour_index + 16 * pal_index as usize, colour);
        }
    }

    /// The DMA register for controlling a single colour in a single background. Good for drawing gradients
    #[must_use]
    fn background_palette_colour_dma(
        &self,
        pal_index: usize,
        colour_index: usize,
    ) -> dma::DmaControllable<Rgb15> {
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
    fn set_background_palette_colour(
        &mut self,
        pal_index: usize,
        colour_index: usize,
        colour: Rgb15,
    ) {
        assert!(pal_index < 16);
        assert!(colour_index < 16);

        PALETTE_BACKGROUND.set(colour_index + 16 * pal_index, colour);
    }

    /// Copies palettes to the background palettes without any checks.
    fn set_background_palettes(&mut self, palettes: &[Palette16]) {
        for (palette_index, entry) in palettes.iter().enumerate() {
            self.set_background_palette(palette_index as u8, entry);
        }
    }

    /// Gets the index of the colour for a given background palette, or None if it doesn't exist
    #[must_use]
    fn find_colour_index_16(&self, palette_index: usize, colour: Rgb15) -> Option<usize> {
        assert!(palette_index < 16);

        (0..16).find(|i| PALETTE_BACKGROUND.get(palette_index * 16 + i) == colour)
    }

    /// Gets the index of the colour in the entire background palette, or None if it doesn't exist
    #[must_use]
    fn find_colour_index_256(&self, colour: Rgb15) -> Option<usize> {
        (0..256).find(|&i| PALETTE_BACKGROUND.get(i) == colour)
    }
}
