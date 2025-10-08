use core::cell::Cell;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use portable_atomic::{AtomicBool, AtomicUsize, Ordering};

use agb::display::GraphicsDist;
use agb::interrupt::{Interrupt, VBlank, add_interrupt_handler};
use embassy_sync::waitqueue::AtomicWaker;

/// VBlank counter
static VBLANK_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// VBlank waker  
static VBLANK_WAKER: AtomicWaker = AtomicWaker::new();

/// Whether the VBlank handler is initialized
static VBLANK_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Initialize VBlank handler
fn init_embassy_vblank() {
    if VBLANK_INITIALIZED.swap(true, Ordering::SeqCst) {
        return; // Already initialized
    }

    // Set up VBlank interrupt handler
    let handler = unsafe {
        add_interrupt_handler(Interrupt::VBlank, |_| {
            VBLANK_COUNTER.store(VBLANK_COUNTER.load(Ordering::SeqCst) + 1, Ordering::SeqCst);
            VBLANK_WAKER.wake();
        })
    };
    core::mem::forget(handler);
}

/// Async wrapper for agb display operations
pub struct AsyncDisplay<'a> {
    graphics: agb::display::Graphics<'a>,
    #[allow(dead_code)]
    vblank: VBlank,
}

impl<'a> AsyncDisplay<'a> {
    pub(crate) fn new(graphics_dist: &'a mut GraphicsDist) -> Self {
        init_embassy_vblank();

        Self {
            graphics: graphics_dist.get(),
            vblank: VBlank::get(),
        }
    }

    /// Wait for the next VBlank interrupt asynchronously
    pub async fn wait_for_vblank(&self) {
        EmbassyVBlankFuture::new().await
    }

    /// Get a frame for rendering, waiting for VBlank if needed
    pub async fn frame(&mut self) -> agb::display::GraphicsFrame<'_> {
        self.wait_for_vblank().await;
        self.graphics.frame()
    }

    /// Get a frame for rendering without waiting for VBlank
    /// Use this when you've already called wait_for_vblank() separately
    pub fn frame_no_wait(&mut self) -> agb::display::GraphicsFrame<'_> {
        self.graphics.frame()
    }

    /// Get access to the underlying graphics for synchronous operations
    pub fn graphics(&mut self) -> &mut agb::display::Graphics<'a> {
        &mut self.graphics
    }
}

/// Future that completes on the next VBlank
struct EmbassyVBlankFuture {
    last_count: Cell<usize>,
}

impl EmbassyVBlankFuture {
    fn new() -> Self {
        Self {
            last_count: Cell::new(VBLANK_COUNTER.load(Ordering::SeqCst)),
        }
    }
}

impl Future for EmbassyVBlankFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let current_count = VBLANK_COUNTER.load(Ordering::SeqCst);
        let last_count = self.last_count.get();

        if current_count > last_count {
            // VBlank occurred since last check
            self.last_count.set(current_count);
            Poll::Ready(())
        } else {
            // Register waker for next VBlank
            VBLANK_WAKER.register(cx.waker());

            // Check again in case VBlank occurred between the first check and waker registration
            let current_count = VBLANK_COUNTER.load(Ordering::SeqCst);
            if current_count > last_count {
                self.last_count.set(current_count);
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        }
    }
}

/// Future for DMA-based transfers (placeholder for future implementation)
pub struct DmaTransferFuture {
    _phantom: core::marker::PhantomData<()>,
}

impl Future for DmaTransferFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        // For now, complete immediately
        // TODO: Implement actual DMA async support
        Poll::Ready(())
    }
}
