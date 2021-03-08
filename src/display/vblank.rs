use super::DISPLAY_STATUS;

#[non_exhaustive]
pub struct VBlankGiver {}

impl VBlankGiver {
    /// Gets a vblank handle where only one can be obtained at a time
    pub fn get(&mut self) -> VBlank {
        unsafe { VBlank::new() }
    }
}

/// Once obtained, this guarentees that interrupts are enabled and set up to
/// allow for waiting for vblank
pub struct VBlank {}

impl VBlank {
    unsafe fn new() -> Self {
        crate::interrupt::enable_interrupts();
        crate::interrupt::enable(crate::interrupt::Interrupt::VBlank);
        enable_VBlank_interrupt();
        VBlank {}
    }

    #[allow(non_snake_case)]
    /// Waits for VBlank using interrupts. This is the preferred method for
    /// waiting until the next frame.
    pub fn wait_for_VBlank(&self) {
        crate::syscall::wait_for_VBlank();
    }
}

impl Drop for VBlank {
    fn drop(&mut self) {
        unsafe {
            disable_VBlank_interrupt();
            crate::interrupt::disable(crate::interrupt::Interrupt::VBlank);
        }
    }
}

#[allow(non_snake_case)]
unsafe fn enable_VBlank_interrupt() {
    let status = DISPLAY_STATUS.get() | (1 << 3);
    DISPLAY_STATUS.set(status);
}

#[allow(non_snake_case)]
unsafe fn disable_VBlank_interrupt() {
    let status = DISPLAY_STATUS.get() & !(1 << 3);
    DISPLAY_STATUS.set(status);
}
