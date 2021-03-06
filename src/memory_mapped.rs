pub struct MemoryMapped<T> {
    address: *mut T,
}

impl<T> MemoryMapped<T> {
    pub const fn new(address: usize) -> Self {
        MemoryMapped {
            address: address as *mut T,
        }
    }

    pub fn get(&self) -> T {
        unsafe { self.address.read_volatile() }
    }

    pub fn set(&self, val: T) {
        unsafe { self.address.write_volatile(val) }
    }
}

pub struct MemoryMapped1DArray<T, const N: usize> {
    array: *mut [T; N],
}

#[allow(dead_code)]
impl<T, const N: usize> MemoryMapped1DArray<T, N> {
    pub const fn new(address: usize) -> Self {
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
}

pub struct MemoryMapped2DArray<T, const X: usize, const Y: usize> {
    array: *mut [[T; X]; Y],
}

impl<T, const X: usize, const Y: usize> MemoryMapped2DArray<T, X, Y> {
    pub const fn new(address: usize) -> Self {
        MemoryMapped2DArray {
            array: address as *mut [[T; X]; Y],
        }
    }
    pub fn get(&self, x: usize, y: usize) -> T {
        unsafe { (&mut (*self.array)[y][x] as *mut T).read_volatile() }
    }
    pub fn set(&self, x: usize, y: usize, val: T) {
        unsafe { (&mut (*self.array)[y][x] as *mut T).write_volatile(val) }
    }
}
