use core::{
    cell::{RefCell, RefMut},
    future::{poll_fn, Future},
    marker::PhantomData,
    pin::Pin,
    ptr::NonNull,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use bitflags::bitflags;

pub mod channel;
mod ringbuf;

use alloc::{boxed::Box, vec::Vec};

use crate::{
    executor::ringbuf::Writer,
    interrupt::{self, Interrupt, InterruptHandler, __RUST_INTERRUPT_HANDLER},
    sync::Static,
    syscall, InternalAllocator,
};

use self::ringbuf::{Reader, RingBuffer};

pub static CURRENT_VBLANK: Static<usize> = Static::new(0);
static INTERRUPTS: RingBuffer<u32, 32> = RingBuffer::new();

/// This only works with the async executor in agb! It avoids the standard waker
/// to use a more efficient waker specifically for the built in executor.
///
/// TODO: Should this be unsafe / can we avoid this being unsound when used with
/// a different executor
pub fn vblank_async() -> impl Future<Output = ()> {
    let current_vblank = CURRENT_VBLANK.read();
    poll_fn(move |c| {
        if CURRENT_VBLANK.read() > current_vblank {
            Poll::Ready(())
        } else {
            let raw_waker = c.waker().as_raw().data();
            add_to_vblank_list(unsafe {
                NonNull::new(raw_waker.cast_mut().cast()).unwrap_unchecked()
            });

            Poll::Pending
        }
    })
}

/// Safety: This function is not reentrant
#[export_name = "__AGBRS_ASYNC_INTERRUPT_HANDLER"]
extern "C" fn async_interrupt_handler(interrupt: u16) -> u16 {
    static mut WRITER: Writer<'static, u32, 32> = unsafe { INTERRUPTS.get_rw_ref().1 };

    let _ = unsafe { &mut WRITER }.try_insert(interrupt as u32);

    __RUST_INTERRUPT_HANDLER(interrupt)
}

#[derive(Default)]
struct ToPoll {
    poll: PollList,
    vblankers: PollList,
}

#[derive(Default)]
struct PollList {
    first: Option<NonNull<Header>>,
    last: Option<NonNull<Header>>,
}

fn add_to_run_list(value: NonNull<Header>) {
    let state = unsafe { (*value.as_ptr()).state };

    if state.intersects(State::QUEUED) {
        return;
    }

    TO_POLL.cell.borrow_mut().poll.add(value);

    unsafe { (*value.as_ptr()).state |= State::QUEUED }
}

fn add_to_vblank_list(value: NonNull<Header>) {
    let state = unsafe { (*value.as_ptr()).state };

    if state.intersects(State::QUEUED) {
        return;
    }

    TO_POLL.cell.borrow_mut().vblankers.add(value);

    unsafe { (*value.as_ptr()).state |= State::QUEUED }
}

impl PollList {
    fn add(&mut self, value: NonNull<Header>) {
        match self.last {
            Some(last) => unsafe { (*last.as_ptr()).next = Some(value) },
            None => {
                self.first = Some(value);
            }
        }
        self.last = Some(value);
        unsafe { (*value.as_ptr()).next = None };
    }

    fn add_list(&mut self, value: NonNull<Header>) {
        match self.last {
            Some(last) => unsafe { (*last.as_ptr()).next = Some(value) },
            None => {
                self.first = Some(value);
            }
        }
        self.last = Some(value);
    }
}

pub struct Executor {
    interrupt_reader: Reader<'static, u32, 32>,
    _vblank_interrupt: InterruptHandler<'static>,
}

bitflags! {
    pub struct State: u32 {
        const QUEUED = 1 << 0;
    }
}

struct Header {
    next: Option<NonNull<Header>>,
    count: usize,
    state: State,
    vtable: &'static TaskVTable,
}

impl Header {
    fn poll(header: NonNull<Header>, ctx: &mut Context) -> Poll<()> {
        let poll = unsafe { header.as_ref() }.vtable.poll;
        unsafe { poll(header, ctx) }
    }
    fn try_read_value(header: NonNull<Header>, dst: *mut (), ctx: &mut Context) {
        let try_read_value = unsafe { header.as_ref() }.vtable.try_read_value;
        unsafe { try_read_value(header, dst, ctx) }
    }
    fn abort(header: NonNull<Header>) {
        let abort = unsafe { header.as_ref() }.vtable.abort;
        unsafe { abort(header) }
    }
    fn is_done(header: NonNull<Header>) -> bool {
        let is_done = unsafe { header.as_ref() }.vtable.is_done;
        unsafe { is_done(header) }
    }
    /// Do not use your header after calling this
    unsafe fn decrement_count(mut header: NonNull<Header>) {
        unsafe { header.as_mut() }.count -= 1;
        let count = unsafe { header.as_mut() }.count;
        if count == 0 {
            let drop = unsafe { header.as_ref() }.vtable.drop;
            drop(header);
        }
    }
}

#[derive(Copy, Clone)]
struct TaskVTable {
    poll: unsafe fn(NonNull<Header>, ctx: *mut Context) -> Poll<()>,
    drop: unsafe fn(NonNull<Header>),
    try_read_value: unsafe fn(NonNull<Header>, dst: *mut (), ctx: *mut Context),
    abort: unsafe fn(NonNull<Header>),
    is_done: unsafe fn(NonNull<Header>) -> bool,
}

impl TaskVTable {
    const fn new<F: Future>() -> &'static TaskVTable {
        unsafe fn poll<F: Future>(head: NonNull<Header>, ctx: *mut Context) -> Poll<()> {
            let task = &mut *head.as_ptr().cast::<TaskCell<F>>();

            let ctx = unsafe { &mut *ctx };

            let task = &mut task.task;

            match &mut task.future {
                Stage::Running(f) => match Pin::new_unchecked(f).poll(ctx) {
                    Poll::Ready(v) => {
                        if let Some(waker) = task.join_waker.take() {
                            waker.wake();
                        }
                        task.future = Stage::Finished(v);
                        Poll::Ready(())
                    }
                    Poll::Pending => Poll::Pending,
                },
                Stage::Finished(_) => Poll::Ready(()),
                Stage::Extracted => Poll::Ready(()),
                Stage::Abort => Poll::Ready(()),
            }
        }

        unsafe fn drop<F: Future>(head: NonNull<Header>) {
            let ptr: NonNull<TaskCell<F>> = head.cast();
            Box::from_raw_in(ptr.as_ptr(), InternalAllocator);
        }

        unsafe fn try_read_value<F: Future>(
            head: NonNull<Header>,
            dst: *mut (),
            ctx: *mut Context,
        ) {
            let task = &mut *head.as_ptr().cast::<TaskCell<F>>();
            let task = &mut task.task;

            let r: *mut Poll<F::Output> = dst.cast();

            let ctx = unsafe { &mut *ctx };

            match task.future {
                Stage::Finished(_) => {
                    match core::mem::replace(&mut task.future, Stage::Extracted) {
                        Stage::Finished(v) => r.write(Poll::Ready(v)),
                        _ => unreachable!(),
                    }
                }
                _ => task.join_waker = Some(ctx.waker().clone()),
            }
        }

        unsafe fn abort<F: Future>(head: NonNull<Header>) {
            let task = &mut *head.as_ptr().cast::<TaskCell<F>>();
            task.task.future = Stage::Abort;
        }

        unsafe fn is_done<F: Future>(head: NonNull<Header>) -> bool {
            let task = &mut *head.as_ptr().cast::<TaskCell<F>>();
            match task.task.future {
                Stage::Running(_) => false,
                Stage::Finished(_) => true,
                Stage::Extracted => true,
                Stage::Abort => true,
            }
        }

        &TaskVTable {
            poll: poll::<F>,
            drop: drop::<F>,
            try_read_value: try_read_value::<F>,
            abort: abort::<F>,
            is_done: is_done::<F>,
        }
    }
}

#[repr(C)]
struct TaskCell<F>
where
    F: Future,
{
    header: Header,
    task: TaskRun<F>,
}

struct TaskRun<F: Future> {
    future: Stage<F>,
    join_waker: Option<Waker>,
}

enum Stage<F: Future> {
    Running(F),
    Finished(F::Output),
    Extracted,
    Abort,
}

struct Task {
    future: NonNull<Header>,
}

pub struct TaskJoin<O> {
    future: NonNull<Header>,
    _phantom: PhantomData<O>,
}

pub struct TaskDropAbort<O> {
    join: TaskJoin<O>,
}

impl<O> Drop for TaskDropAbort<O> {
    fn drop(&mut self) {
        self.join.abort();
    }
}

impl<O> Future for TaskJoin<O> {
    type Output = O;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut ret = Poll::Pending;

        Header::try_read_value(self.future, &mut ret as *mut _ as *mut _, cx);

        ret
    }
}

impl<O> TaskJoin<O> {
    pub fn abort(&self) {
        Header::abort(self.future);
    }

    #[must_use]
    pub fn abort_on_drop(self) -> TaskDropAbort<O> {
        TaskDropAbort { join: self }
    }

    #[must_use]
    pub fn is_done(&self) -> bool {
        Header::is_done(self.future)
    }
}

impl<O> Drop for TaskJoin<O> {
    fn drop(&mut self) {
        unsafe { Header::decrement_count(self.future) }
    }
}

impl Task {
    /// You must ensure future lives long enough
    unsafe fn new<F>(future: F) -> (Task, TaskJoin<F::Output>)
    where
        F: Future,
    {
        let task = TaskCell {
            header: Header {
                next: None,
                count: 2,
                state: State::empty(),
                vtable: TaskVTable::new::<F>(),
            },
            task: TaskRun {
                future: Stage::Running(future),
                join_waker: None,
            },
        };

        let boxed = Box::new_in(task, InternalAllocator);
        let leaked = Box::into_raw(boxed);
        let erased = NonNull::new(leaked).unwrap().cast();

        (
            Task { future: erased },
            TaskJoin {
                future: erased,
                _phantom: PhantomData,
            },
        )
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        Header::poll(self.future, context)
    }
}

impl Executor {
    unsafe fn new() -> Self {
        let (reader, _) = INTERRUPTS.get_rw_ref();

        Executor {
            interrupt_reader: reader,
            _vblank_interrupt: interrupt::add_interrupt_handler(Interrupt::VBlank, |_| {
                CURRENT_VBLANK.write(CURRENT_VBLANK.read() + 1);
            }),
        }
    }
}

unsafe fn get_raw_waker(value: NonNull<Header>) -> RawWaker {
    fn clone(data: *const ()) -> RawWaker {
        unsafe { get_raw_waker(NonNull::new(data.cast_mut().cast()).unwrap()) }
    }

    fn wake(data: *const ()) {
        add_to_run_list(NonNull::new(data.cast_mut().cast()).unwrap());
    }

    fn wake_by_ref(data: *const ()) {
        add_to_run_list(NonNull::new(data.cast_mut().cast()).unwrap());
    }

    fn drop(_data: *const ()) {}

    let vtable = &RawWakerVTable::new(clone, wake, wake_by_ref, drop);

    RawWaker::new(value.as_ptr() as *const _ as *const _, vtable)
}

unsafe fn get_waker(value: NonNull<Header>) -> Waker {
    unsafe { Waker::from_raw(get_raw_waker(value)) }
}

static TO_POLL: PromiseSingleThreadRefCell<ToPoll> = PromiseSingleThreadRefCell {
    cell: RefCell::new(ToPoll {
        poll: PollList {
            first: None,
            last: None,
        },
        vblankers: PollList {
            first: None,
            last: None,
        },
    }),
};

impl Executor {
    fn get_to_poll(&mut self) -> Option<PollList> {
        let mut poll = TO_POLL.cell.borrow_mut();

        poll.poll.first?;

        let mut to_poll = Default::default();
        core::mem::swap(&mut poll.poll, &mut to_poll);

        Some(to_poll)
    }

    fn run(&mut self) -> ! {
        loop {
            self.add_waiting();

            while let Some(mut to_poll) = self.get_to_poll() {
                let mut next = to_poll.first.take();

                while let Some(mut header) = next {
                    let waker = unsafe { get_waker(header) };
                    let to_be_next = unsafe { header.as_mut().next.take() };

                    let mut ctx = Context::from_waker(&waker);

                    unsafe {
                        (*header.as_ptr()).state.remove(State::QUEUED);
                    }

                    match Header::poll(header, &mut ctx) {
                        Poll::Ready(_) => unsafe {
                            Header::decrement_count(header);
                        },
                        Poll::Pending => {}
                    }

                    next = to_be_next;
                }

                self.add_waiting();
                self.process_interrupts();
            }

            syscall::halt();

            self.process_interrupts();
        }
    }

    fn add_waiting(&mut self) {
        for task in TO_ADD_TO_EXECUTOR.cell.borrow_mut().drain(..) {
            add_to_run_list(task.future);
        }
    }

    fn process_interrupts(&mut self) {
        while let Some(interrupts) = self.interrupt_reader.try_read() {
            if interrupts & (1 << Interrupt::VBlank as u32) != 0 {
                let poll = TO_POLL.cell.borrow_mut();
                let (mut poll, mut vblank) =
                    RefMut::map_split(poll, |f| (&mut f.poll, &mut f.vblankers));

                let list = vblank.first.take();
                if let Some(list) = list {
                    poll.add_list(list);
                }
                vblank.last.take();
            }
        }
    }
}

struct PromiseSingleThreadRefCell<T> {
    cell: RefCell<T>,
}

unsafe impl<T> Sync for PromiseSingleThreadRefCell<T> {}

static TO_ADD_TO_EXECUTOR: PromiseSingleThreadRefCell<Vec<Task>> = PromiseSingleThreadRefCell {
    cell: RefCell::new(Vec::new()),
};

pub fn async_main<F, Fut>(gba: crate::Gba, future: F) -> !
where
    F: FnOnce(crate::Gba) -> Fut,
    Fut: Future + 'static,
{
    let (task, _) = unsafe { Task::new(future(gba)) };
    TO_ADD_TO_EXECUTOR.cell.borrow_mut().push(task);

    unsafe { Executor::new().run() }
}

pub fn spawn<F>(future: F) -> TaskJoin<F::Output>
where
    F: Future + 'static,
{
    let (task, join) = unsafe { Task::new(future) };
    TO_ADD_TO_EXECUTOR.cell.borrow_mut().push(task);

    join
}

pub fn yeild() -> impl Future<Output = ()> {
    let mut has_polled = false;

    poll_fn(move |c| {
        if has_polled {
            Poll::Ready(())
        } else {
            has_polled = true;
            c.waker().wake_by_ref();
            Poll::Pending
        }
    })
}

pub fn suspend() -> impl Future<Output = ()> {
    poll_fn(|_| Poll::Pending)
}
