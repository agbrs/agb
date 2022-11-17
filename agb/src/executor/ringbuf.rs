use core::{cell::UnsafeCell, mem::MaybeUninit};

use crate::sync::Static;

pub struct RingBuffer<T, const N: usize> {
    write_head: Static<usize>,
    read_head: Static<usize>,
    data: [UnsafeCell<MaybeUninit<T>>; N],
}

unsafe impl<T, const N: usize> Send for RingBuffer<T, N> {}
unsafe impl<T, const N: usize> Sync for RingBuffer<T, N> {}

impl<T, const N: usize> RingBuffer<T, N> {
    const ELEMENT: UnsafeCell<MaybeUninit<T>> = UnsafeCell::new(MaybeUninit::uninit());

    pub const fn new() -> Self {
        RingBuffer {
            write_head: Static::new(0),
            read_head: Static::new(0),
            data: [Self::ELEMENT; N],
        }
    }

    #[cfg(test)]
    pub fn get_rw(&mut self) -> (Reader<T, N>, Writer<T, N>) {
        (Reader { buf: self }, Writer { buf: self })
    }

    /// # Safety:
    /// You must only have one reader and one writer
    pub const unsafe fn get_rw_ref(&self) -> (Reader<T, N>, Writer<T, N>) {
        (Reader { buf: self }, Writer { buf: self })
    }
}

pub struct Writer<'a, T, const N: usize> {
    buf: &'a RingBuffer<T, N>,
}

pub struct Reader<'a, T, const N: usize> {
    buf: &'a RingBuffer<T, N>,
}

fn mod_power_of_2(left: usize, right: usize) -> usize {
    left & (right - 1)
}

pub enum BufError {
    Full,
}

impl<'a, T, const N: usize> Writer<'a, T, N> {
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

impl<'a, T, const N: usize> Reader<'a, T, N> {
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
        let mut buf: RingBuffer<_, 16> = RingBuffer::new();

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
