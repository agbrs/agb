use core::ops;

pub struct MemoryMapped<T> {
    address: *mut T,
}

impl<T> MemoryMapped<T> {
    pub const unsafe fn new(address: usize) -> Self {
        MemoryMapped {
            address: address as *mut T,
        }
    }

    pub fn get(&self) -> T {
        unsafe { self.address.read_volatile() }
    }

    pub fn set(&self, val: T) {
        if core::mem::size_of::<T>() != 0 {
            unsafe { self.address.write_volatile(val) }
        }
    }
}

impl<T> MemoryMapped<T>
where
    T: From<u8>
        + Copy
        + ops::Shl<Output = T>
        + ops::BitAnd<Output = T>
        + ops::Sub<Output = T>
        + ops::BitOr<Output = T>
        + ops::Not<Output = T>,
{
    pub fn set_bits(&self, value: T, length: T, shift: T) {
        let one: T = 1u8.into();
        let mask: T = (one << length) - one;
        let current_val = self.get();
        self.set((current_val & !(mask << shift)) | ((value & mask) << shift));
    }
}

pub fn set_bits<T>(current_value: T, value: T, length: usize, shift: usize) -> T
where
    T: From<u8>
        + Copy
        + ops::Shl<usize, Output = T>
        + ops::BitAnd<Output = T>
        + ops::Sub<Output = T>
        + ops::BitOr<Output = T>
        + ops::Not<Output = T>,
{
    let one: T = 1u8.into();
    let mask: T = (one << length) - one;
    (current_value & !(mask << shift)) | ((value & mask) << shift)
}

pub struct MemoryMapped1DArray<T, const N: usize> {
    array: *mut [T; N],
}

#[allow(dead_code)]
impl<T, const N: usize> MemoryMapped1DArray<T, N> {
    pub const unsafe fn new(address: usize) -> Self {
        MemoryMapped1DArray {
            array: address as *mut [T; N],
        }
    }

    pub fn get(&self, n: usize) -> T {
        unsafe { (&mut (*self.array)[n] as *mut T).read_volatile() }
    }

    pub fn set(&self, n: usize, val: T) {
        unsafe { (&mut (*self.array)[n] as *mut T).write_volatile(val) }
    }

    pub fn as_ptr(&self) -> *mut T {
        self.array.cast()
    }
}
