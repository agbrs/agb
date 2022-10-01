use crate::memory_mapped::MemoryMapped;

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

    crate::interrupt::free(|_| {
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