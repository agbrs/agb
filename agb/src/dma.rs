use core::{
    mem::{MaybeUninit, size_of},
    pin::Pin,
};

use alloc::boxed::Box;

use crate::{
    display::{DmaFrame, GraphicsFrame},
    memory_mapped::MemoryMapped,
};

pub(crate) struct Dma {
    number: usize,

    source_addr: MemoryMapped<u32>,
    dest_addr: MemoryMapped<u32>,
    ctrl_addr: MemoryMapped<u32>,
}

impl Dma {
    pub(crate) unsafe fn new(number: usize) -> Self {
        Self {
            number,
            source_addr: unsafe { MemoryMapped::new(dma_source_addr(number)) },
            dest_addr: unsafe { MemoryMapped::new(dma_dest_addr(number)) },
            ctrl_addr: unsafe { MemoryMapped::new(dma_control_addr(number)) },
        }
    }

    pub(crate) fn disable(&mut self) {
        unsafe { MemoryMapped::new(dma_control_addr(self.number)) }.set(0);
    }
}

/// A struct to describe things you can modify using DMA (normally some register within the GBA)
///
/// This is generally used to perform fancy graphics tricks like screen wobble on a per-scanline basis or
/// to be able to create a track like in mario kart. This is an advanced technique and likely not needed
/// unless you want to do fancy graphics.
pub struct DmaControllable<Item> {
    memory_location: *mut Item,
}

impl<Item> DmaControllable<Item> {
    pub(crate) unsafe fn new(memory_location: *mut Item) -> Self {
        Self { memory_location }
    }
}

pub struct HBlankDmaDefinition<Item> {
    controllable: DmaControllable<Item>,
    values: Pin<Box<[Item; 161]>>,
}

impl<Item> HBlankDmaDefinition<Item>
where
    Item: Copy + Unpin + 'static,
{
    pub fn new(controllable: DmaControllable<Item>, values: &[Item]) -> Self {
        assert!(
            values.len() >= 160,
            "need to pass at least 160 values for a hblank transfer"
        );

        let mut copied_values = Box::into_pin(Box::new([const { MaybeUninit::uninit() }; 161]));
        copied_values[..160].copy_from_slice(unsafe {
            core::mem::transmute::<&[Item], &[MaybeUninit<Item>]>(&values[..160])
        });

        copied_values[160].write(values[0]);

        Self {
            controllable,
            values: unsafe {
                core::mem::transmute::<Pin<Box<[MaybeUninit<Item>; 161]>>, Pin<Box<[Item; 161]>>>(
                    copied_values,
                )
            },
        }
    }

    pub fn show(self, frame: &mut GraphicsFrame) {
        frame.add_dma(self);
    }
}

impl<Item> DmaFrame for HBlankDmaDefinition<Item>
where
    Item: Copy + 'static,
{
    fn commit(&mut self) {
        let dma = unsafe { Dma::new(0) };

        let n_transfers = (size_of::<Item>() / 2) as u32;

        dma.source_addr.set(self.values[1..].as_ptr() as u32);
        dma.dest_addr.set(self.controllable.memory_location as u32);

        unsafe {
            self.controllable
                .memory_location
                .write_volatile(self.values[0]);
        }

        dma.ctrl_addr.set(
            (0b11 << 0x15) | // keep destination address fixed
            // (0b00 << 0x17) | // increment the source address each time
            (1 << 0x19) | // repeat the copy each hblank
            // 0 << 0x1a | // copy in half words (see n_transfers above)
            (0b10 << 0x1c) | // copy each hblank
            (1 << 0x1f) | // enable the dma
            n_transfers, // the number of halfwords to copy
        );
    }

    fn cleanup(&mut self) {
        unsafe { Dma::new(0) }.disable();
    }
}

const fn dma_source_addr(dma: usize) -> usize {
    0x0400_00b0 + 0x0c * dma
}

const fn dma_dest_addr(dma: usize) -> usize {
    0x0400_00b4 + 0x0c * dma
}

const fn dma_control_addr(dma: usize) -> usize {
    0x0400_00b8 + 0x0c * dma
}

const DMA3_SOURCE_ADDR: MemoryMapped<u32> = unsafe { MemoryMapped::new(dma_source_addr(3)) };
const DMA3_DEST_ADDR: MemoryMapped<u32> = unsafe { MemoryMapped::new(dma_dest_addr(3)) };
const DMA3_CONTROL: MemoryMapped<u32> = unsafe { MemoryMapped::new(dma_control_addr(3)) };

pub(crate) unsafe fn dma_copy16(src: *const u16, dest: *mut u16, count: usize) {
    assert!(count < u16::MAX as usize);

    DMA3_SOURCE_ADDR.set(src as u32);
    DMA3_DEST_ADDR.set(dest as u32);

    DMA3_CONTROL.set(count as u32 | (1 << 31));
}

pub(crate) fn dma3_exclusive<R>(f: impl FnOnce() -> R) -> R {
    const DMA0_CTRL_HI: MemoryMapped<u16> = unsafe { MemoryMapped::new(dma_control_addr(0) + 2) };
    const DMA1_CTRL_HI: MemoryMapped<u16> = unsafe { MemoryMapped::new(dma_control_addr(1) + 2) };
    const DMA2_CTRL_HI: MemoryMapped<u16> = unsafe { MemoryMapped::new(dma_control_addr(2) + 2) };

    critical_section::with(|_| {
        let dma0_ctl = DMA0_CTRL_HI.get();
        let dma1_ctl = DMA1_CTRL_HI.get();
        let dma2_ctl = DMA2_CTRL_HI.get();
        DMA0_CTRL_HI.set(dma0_ctl & !(1 << 15));
        DMA1_CTRL_HI.set(dma1_ctl & !(1 << 15));
        DMA2_CTRL_HI.set(dma2_ctl & !(1 << 15));

        // Executes the body of the function with DMAs and IRQs disabled.
        let ret = f();

        // Continues higher priority DMAs if they were enabled before.
        DMA0_CTRL_HI.set(dma0_ctl);
        DMA1_CTRL_HI.set(dma1_ctl);
        DMA2_CTRL_HI.set(dma2_ctl);

        // returns the return value
        ret
    })
}
