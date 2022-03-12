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

pub(crate) unsafe fn dma_copy(src: *const u16, dest: *mut u16, count: usize) {
    assert!(count < u16::MAX as usize);

    DMA3_SOURCE_ADDR.set(src as u32);
    DMA3_DEST_ADDR.set(dest as u32);

    DMA3_CONTROL.set(count as u32 | (1 << 31));
}
