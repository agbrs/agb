use core::{
    cell::UnsafeCell,
    future::{poll_fn, Future},
    mem::MaybeUninit,
    ops::DerefMut,
    task::{Poll, Waker},
};

use alloc::{boxed::Box, rc::Rc, vec::Vec};

/// This is implemented using a read head and a length. This avoids wasting a
/// slot in the backing array due to no ambiguity between full and empty.
/// This works in single threaded land (and is not interrupt safe, which the ringbuf is).
struct Inner<T> {
    read_head: usize,
    length: usize,
    read_waker: Option<Waker>,
    write_waker: Option<Waker>,
    data: Box<[MaybeUninit<T>]>,
}

fn mod_power_of_2(left: usize, right: usize) -> usize {
    left & (right - 1)
}

impl<T> Inner<T> {
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

impl<T> Drop for Inner<T> {
    fn drop(&mut self) {
        for i in 0..self.length {
            unsafe { self.data[self.read_head + i].assume_init_drop() }
        }
    }
}

pub struct Reader<T> {
    inner: Rc<UnsafeCell<Inner<T>>>,
}

pub struct Writer<T> {
    inner: Rc<UnsafeCell<Inner<T>>>,
}

#[must_use]
pub fn new_with_capacity<T>(capacity: usize) -> (Reader<T>, Writer<T>) {
    assert!(
        capacity.is_power_of_two(),
        "capacity should be a power of 2"
    );

    let mut storage = Vec::with_capacity(capacity);

    for _ in 0..capacity {
        storage.push(MaybeUninit::uninit());
    }

    let inner = Inner {
        read_head: 0,
        length: 0,
        read_waker: None,
        write_waker: None,
        data: storage.into_boxed_slice(),
    };
    let inner = Rc::new(UnsafeCell::new(inner));

    (
        Reader {
            inner: inner.clone(),
        },
        Writer { inner },
    )
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum ChannelError {
    Closed,
    Full,
}

impl<T> Drop for Writer<T> {
    fn drop(&mut self) {
        if let Some(waker) = unsafe { self.inner().read_waker.take() } {
            waker.wake();
        }
    }
}

impl<T> Writer<T> {
    unsafe fn inner(&self) -> impl DerefMut<Target = Inner<T>> + '_ {
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
}

pub struct WriterFuture<'a, T> {
    value_to_write: UnsafeCell<Option<T>>,
    inner: &'a Rc<UnsafeCell<Inner<T>>>,
}

impl<'a, T> Future for WriterFuture<'a, T> {
    type Output = Result<(), ChannelError>;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> Poll<Self::Output> {
        let strong_count = Rc::strong_count(self.inner);
        let inner = unsafe { &mut *self.inner.get() };

        if inner.is_full() {
            inner.write_waker = Some(cx.waker().clone());
            return Poll::Pending;
        }
        if strong_count == 1 {
            return Poll::Ready(Err(ChannelError::Closed));
        }
        if let Some(value) = unsafe { &mut *self.value_to_write.get() }.take() {
            unsafe { inner.write_assume_not_full(value) };
            if let Some(waker) = inner.read_waker.take() {
                waker.wake();
            }
        }

        Poll::Ready(Ok(()))
    }
}

impl<T: Clone> Writer<T> {
    pub fn write(&mut self, value: T) -> WriterFuture<'_, T> {
        WriterFuture {
            value_to_write: UnsafeCell::new(Some(value)),
            inner: &self.inner,
        }
    }
}

impl<T> Reader<T> {
    unsafe fn inner(&self) -> impl DerefMut<Target = Inner<T>> + '_ {
        &mut *self.inner.get()
    }

    /// Reads from the channel or waits until there is data in the channel
    pub fn read(&mut self) -> impl Future<Output = Result<T, ChannelError>> + '_ {
        poll_fn(move |cx| {
            match self.read_immediate() {
                Ok(v) => {
                    if let Some(v) = v {
                        return Poll::Ready(Ok(v));
                    }
                }
                Err(err) => return Poll::Ready(Err(err)),
            };

            let mut inner = unsafe { self.inner() };

            inner.read_waker = Some(cx.waker().clone());
            Poll::Pending
        })
    }

    /// Immediately reads a value from the channel or None if the channel is empty.
    pub fn read_immediate(&mut self) -> Result<Option<T>, ChannelError> {
        let mut inner = unsafe { self.inner() };

        match inner.read() {
            Some(x) => {
                if let Some(waker) = inner.write_waker.take() {
                    waker.wake();
                }
                Ok(Some(x))
            }
            None => {
                if Rc::strong_count(&self.inner) == 1 {
                    Err(ChannelError::Closed)
                } else {
                    Ok(None)
                }
            }
        }
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
