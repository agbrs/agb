use core::marker::PhantomData;

pub use embassy_executor::Spawner;
use embassy_executor::raw;

/// Embassy executor for GBA using spin-based polling
pub struct Executor {
    inner: raw::Executor,
    not_send: PhantomData<*mut ()>,
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

impl Executor {
    /// Create a new executor for GBA
    pub fn new() -> Self {
        Self {
            inner: raw::Executor::new(core::ptr::null_mut()),
            not_send: PhantomData,
        }
    }

    /// Run the executor with the given initialization function
    ///
    /// This function never returns and will run the async main loop.
    /// The init closure receives a Spawner that can be used to spawn initial tasks.
    pub fn run(&'static mut self, init: impl FnOnce(Spawner)) -> ! {
        // Initialize time driver if enabled
        #[cfg(feature = "_time-driver")]
        crate::time_driver::init();

        // Call the init function with our spawner
        init(self.inner.spawner());

        // Main executor loop - poll tasks continuously
        loop {
            unsafe {
                self.inner.poll();
            }

            // Use agb's halt to save power when no tasks are ready
            // This is more power efficient than busy-waiting
            agb::halt();
        }
    }
}
