use core::{alloc::Allocator, cell::UnsafeCell, mem::MaybeUninit};

use alloc::{alloc::Global, boxed::Box, vec::Vec};

use crate::sync::Static;

pub struct RingBufferBox<T, A: Allocator = Global> {
    write_head: Static<usize>,
    read_head: Static<usize>,
    data: Box<[UnsafeCell<MaybeUninit<T>>], A>,
}

impl<T> RingBufferBox<T> {
    pub fn new(capacity: usize) -> RingBufferBox<T> {
        RingBufferBox::new_in(capacity, Global)
    }
}

unsafe impl<T, A: Allocator> Send for RingBufferBox<T, A> {}
unsafe impl<T, A: Allocator> Sync for RingBufferBox<T, A> {}

impl<T, A: Allocator> RingBufferBox<T, A> {
    pub fn new_in(capacity: usize, allocator: A) -> Self {
        assert!(capacity.is_power_of_two());

        let mut storage = Vec::with_capacity_in(capacity, allocator);

        for _ in 0..capacity {
            storage.push(UnsafeCell::new(MaybeUninit::uninit()));
        }

        RingBufferBox {
            write_head: Static::new(0),
            read_head: Static::new(0),
            data: storage.into_boxed_slice(),
        }
    }

    pub fn get_rw(&mut self) -> (Reader<T, A>, Writer<T, A>) {
        (Reader { buf: self }, Writer { buf: self })
    }
}

fn mod_power_of_2(left: usize, right: usize) -> usize {
    left & (right - 1)
}

pub struct Writer<'a, T, A: Allocator = Global> {
    buf: &'a RingBufferBox<T, A>,
}

pub enum BufError {
    Full,
}

impl<'a, T, A: Allocator> Writer<'a, T, A> {
    pub fn try_insert(&mut self, value: T) -> Result<(), BufError> {
        let tip = self.buf.write_head.read();
        let tail = self.buf.read_head.read();
        let next = mod_power_of_2(tip + 1, self.buf.data.len());

        if tail == next {
            return Err(BufError::Full);
        }

        unsafe {
            (*self.buf.data[tip].get()).write(value);
        }

        self.buf.write_head.write(next);

        Ok(())
    }
}

pub struct Reader<'a, T, A: Allocator = Global> {
    buf: &'a RingBufferBox<T, A>,
}

impl<'a, T, A: Allocator> Reader<'a, T, A> {
    pub fn try_read(&mut self) -> Option<T> {
        let read_head = self.buf.read_head.read();
        let write_head = self.buf.write_head.read();

        if read_head != write_head {
            let data = unsafe { (*self.buf.data[read_head].get()).assume_init_read() };

            let next_read = mod_power_of_2(read_head + 1, self.buf.data.len());
            self.buf.read_head.write(next_read);

            Some(data)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test_case]
    fn check_adding_and_reading(_: &mut crate::Gba) {
        let mut buf = RingBufferBox::new(16);

        let (mut reader, mut writer) = buf.get_rw();

        assert_eq!(reader.try_read(), None);

        assert!(writer.try_insert(1).is_ok());

        assert_eq!(reader.try_read(), Some(1));

        assert!(writer.try_insert(2).is_ok());
        assert!(writer.try_insert(3).is_ok());

        assert_eq!(reader.try_read(), Some(2));
        assert_eq!(reader.try_read(), Some(3));
        assert_eq!(reader.try_read(), None);

        for i in 0..15 {
            assert!(writer.try_insert(i).is_ok());
        }

        assert!(writer.try_insert(15).is_err());
    }
}
