use core::cell::UnsafeCell;
use critical_section::Mutex;

/// Internal storage for the agb::Gba instance
/// This is used by the macro system to store the Gba instance globally
static GBA_INSTANCE: Mutex<UnsafeCell<Option<agb::Gba>>> = Mutex::new(UnsafeCell::new(None));

/// Set the global agb instance (used by macros, do not call directly)
#[doc(hidden)]
pub unsafe fn set_agb_instance(gba: agb::Gba) {
    critical_section::with(|cs| unsafe {
        *GBA_INSTANCE.borrow(cs).get() = Some(gba);
    });
}

/// Get the global agb instance (used internally, do not call directly)
#[doc(hidden)]
pub unsafe fn get_agb_instance() -> &'static mut agb::Gba {
    critical_section::with(|cs| unsafe {
        (*GBA_INSTANCE.borrow(cs).get())
            .as_mut()
            .expect("agb instance not set")
    })
}
