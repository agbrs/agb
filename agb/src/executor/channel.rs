use core::{
    alloc::Allocator,
    cell::UnsafeCell,
    future::{poll_fn, Future},
    mem::MaybeUninit,
    ops::DerefMut,
    task::{Poll, Waker},
};

use alloc::{alloc::Global, boxed::Box, vec::Vec};

/// This is implemented using a read head and a length. This avoids wasting a
/// slot in the backing array due to no ambiguity between full and empty.
/// This works in single threaded land (and is not interrupt safe, which the ringbuf is).
struct Inner<T, A: Allocator = Global> {
    read_head: usize,
    length: usize,
    read_waker: Option<Waker>,
    write_waker: Option<Waker>,
    count: usize,
    data: Box<[MaybeUninit<T>], A>,
}

pub struct Channel<T, A: Allocator = Global> {
    inner: UnsafeCell<Inner<T, A>>,
}

fn mod_power_of_2(left: usize, right: usize) -> usize {
    left & (right - 1)
}

impl<T, A: Allocator> Inner<T, A> {
    fn is_empty(&self) -> bool {
        self.length == 0
    }

    fn is_full(&self) -> bool {
        self.length == self.data.len()
    }

    fn read(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            self.length -= 1;

            let data = unsafe { self.data[self.read_head].assume_init_read() };
            self.read_head = mod_power_of_2(self.read_head + 1, self.data.len());
            Some(data)
        }
    }

    fn read_immediate(&mut self) -> Result<Option<T>, ChannelError> {
        match self.read() {
            Some(x) => {
                if let Some(waker) = self.write_waker.take() {
                    waker.wake();
                }
                Ok(Some(x))
            }
            None => {
                if self.count == 1 {
                    Err(ChannelError::Closed)
                } else {
                    Ok(None)
                }
            }
        }
    }

    fn write(&mut self, value: T) -> Result<(), ChannelError> {
        if self.is_full() {
            Err(ChannelError::Full)
        } else {
            unsafe { self.write_assume_not_full(value) };

            Ok(())
        }
    }

    unsafe fn write_assume_not_full(&mut self, value: T) {
        self.data[mod_power_of_2(self.read_head + self.length, self.data.len())].write(value);
        self.length += 1;
    }
}

impl<T, A: Allocator> Drop for Inner<T, A> {
    fn drop(&mut self) {
        for i in 0..self.length {
            unsafe { self.data[self.read_head + i].assume_init_drop() }
        }
    }
}

pub struct Reader<'channel, T, A: Allocator> {
    inner: &'channel UnsafeCell<Inner<T, A>>,
}

pub struct Writer<'channel, T, A: Allocator> {
    inner: &'channel UnsafeCell<Inner<T, A>>,
}

#[must_use]
pub fn new_with_capacity_in<T, A: Allocator>(capacity: usize, allocator: A) -> Channel<T, A> {
    assert!(
        capacity.is_power_of_two(),
        "capacity should be a power of 2"
    );

    let mut storage = Vec::with_capacity_in(capacity, allocator);

    for _ in 0..capacity {
        storage.push(MaybeUninit::uninit());
    }

    let inner = Inner {
        read_head: 0,
        length: 0,
        read_waker: None,
        write_waker: None,
        count: 0,
        data: storage.into_boxed_slice(),
    };
    let inner = UnsafeCell::new(inner);

    Channel { inner }
}

#[must_use]
pub fn new_with_capacity<T>(capacity: usize) -> Channel<T> {
    new_with_capacity_in(capacity, Global)
}

impl<T, A: Allocator> Channel<T, A> {
    pub fn get_reader_writer(&mut self) -> (Reader<T, A>, Writer<T, A>) {
        let inner = self.inner.get_mut();
        inner.count += 2;

        (Reader { inner: &self.inner }, Writer { inner: &self.inner })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum ChannelError {
    Closed,
    Full,
}

impl<'ch, T, A: Allocator> Drop for Writer<'ch, T, A> {
    fn drop(&mut self) {
        let mut inner = unsafe { self.inner() };
        if let Some(waker) = inner.read_waker.take() {
            waker.wake();
        }
        inner.count -= 1;
    }
}

impl<'ch, T, A: Allocator> Writer<'ch, T, A> {
    unsafe fn inner(&self) -> impl DerefMut<Target = Inner<T, A>> + '_ {
        &mut *self.inner.get()
    }

    pub fn write_immediate(&mut self, value: T) -> Result<(), ChannelError> {
        let mut inner = unsafe { self.inner() };
        if let Some(waker) = inner.read_waker.take() {
            waker.wake();
        }
        inner.write(value)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        unsafe { self.inner() }.is_empty()
    }

    #[must_use]
    pub fn is_full(&self) -> bool {
        unsafe { self.inner() }.is_full()
    }

    pub fn write(&mut self, value: T) -> impl Future<Output = Result<(), ChannelError>> + 'ch {
        let mut val = Some(value);
        let inner = self.inner;

        poll_fn(move |cx| {
            let inner = unsafe { &mut *inner.get() };
            if inner.count == 1 {
                return Poll::Ready(Err(ChannelError::Closed));
            }
            if inner.is_full() {
                inner.write_waker = Some(cx.waker().clone());
                return Poll::Pending;
            }

            unsafe {
                inner.write_assume_not_full(val.take().expect("should not poll after completing"));
            }

            if let Some(waker) = inner.read_waker.take() {
                waker.wake();
            }

            Poll::Ready(Ok(()))
        })
    }
}

impl<'ch, T, A: Allocator> Reader<'ch, T, A> {
    unsafe fn inner(&self) -> impl DerefMut<Target = Inner<T, A>> + '_ {
        &mut *self.inner.get()
    }

    /// Reads from the channel or waits until there is data in the channel
    pub fn read(&mut self) -> impl Future<Output = Result<T, ChannelError>> + 'ch {
        let inner = self.inner;

        poll_fn(move |cx| {
            let inner = unsafe { &mut *inner.get() };

            match inner.read_immediate() {
                Ok(v) => {
                    if let Some(v) = v {
                        return Poll::Ready(Ok(v));
                    }
                }
                Err(err) => return Poll::Ready(Err(err)),
            };

            inner.read_waker = Some(cx.waker().clone());
            Poll::Pending
        })
    }

    /// Immediately reads a value from the channel or None if the channel is empty.
    pub fn read_immediate(&mut self) -> Result<Option<T>, ChannelError> {
        let mut inner = unsafe { self.inner() };
        inner.read_immediate()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        unsafe { self.inner().is_empty() }
    }

    #[must_use]
    pub fn is_full(&self) -> bool {
        unsafe { self.inner().is_full() }
    }
}

impl<'ch, T, A: Allocator> Drop for Reader<'ch, T, A> {
    fn drop(&mut self) {
        let mut inner = unsafe { self.inner() };
        if let Some(waker) = inner.write_waker.take() {
            waker.wake();
        }
        inner.count -= 1;
    }
}
