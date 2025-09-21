use core::cell::Cell;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use agb::display::GraphicsDist;
use agb::interrupt::VBlank;

/// Async wrapper for agb display operations
pub struct AsyncDisplay<'a> {
    graphics: agb::display::Graphics<'a>,
    vblank: VBlank,
}

impl<'a> AsyncDisplay<'a> {
    pub(crate) fn new(graphics_dist: &'a mut GraphicsDist) -> Self {
        Self {
            graphics: graphics_dist.get(),
            vblank: VBlank::get(),
        }
    }

    /// Wait for the next VBlank interrupt asynchronously
    pub async fn wait_for_vblank(&self) {
        VBlankFuture::new(&self.vblank).await
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
struct VBlankFuture<'a> {
    vblank: &'a VBlank,
    initialized: Cell<bool>,
}

impl<'a> VBlankFuture<'a> {
    fn new(vblank: &'a VBlank) -> Self {
        Self {
            vblank,
            initialized: Cell::new(false),
        }
    }
}

impl<'a> Future for VBlankFuture<'a> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.initialized.get() {
            // First poll: initialize and check if VBlank already occurred
            self.initialized.set(true);

            // Check if VBlank has already occurred since last time
            if self.vblank.has_vblank_occurred() {
                return Poll::Ready(());
            }

            // No VBlank yet, schedule a wake-up and return pending
            cx.waker().wake_by_ref();
            Poll::Pending
        } else {
            // Subsequent polls: check for VBlank non-blocking
            if self.vblank.has_vblank_occurred() {
                Poll::Ready(())
            } else {
                // Still no VBlank, schedule another wake-up
                cx.waker().wake_by_ref();
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
