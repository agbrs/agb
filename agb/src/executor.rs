use core::{
    cell::{RefCell, RefMut},
    future::{poll_fn, Future},
    marker::PhantomData,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    pin::Pin,
    ptr::NonNull,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

pub mod channel;
mod ringbox;
mod ringbuf;

use alloc::{rc::Rc, vec::Vec};
use slotmap::DefaultKey;

use crate::{
    executor::ringbuf::Writer,
    interrupt::{self, Interrupt, InterruptHandler, __RUST_INTERRUPT_HANDLER},
    sync::Static,
    syscall,
};

use self::ringbuf::{Reader, RingBuffer};

static CURRENT_VBLANK: Static<usize> = Static::new(0);
static INTERRUPTS: RingBuffer<u32, 32> = RingBuffer::new();

struct DebugRefCell<T> {
    cell: core::cell::UnsafeCell<T>,
}

impl<T> DebugRefCell<T> {
    fn new(t: T) -> Self {
        DebugRefCell {
            cell: core::cell::UnsafeCell::new(t),
        }
    }

    fn get(&'_ self) -> NonNull<T> {
        // # Safety
        // I'm pretty sure theres some guarentee that an unsafe cell always gives a non null pointer.
        unsafe { NonNull::new(self.cell.get()).unwrap_unchecked() }
    }
}

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
            let task_waker = unsafe { TaskWaker::from_ptr(raw_waker) };
            task_waker
                .to_poll
                .borrow_mut()
                .vblank_waiting
                .push(task_waker.id);
            core::mem::forget(task_waker);

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
    poll: Vec<DefaultKey>,
    vblank_waiting: Vec<DefaultKey>,
}

pub struct Executor {
    futures: slotmap::SlotMap<DefaultKey, Task>,
    to_poll: Rc<RefCell<ToPoll>>,
    interrupt_reader: Reader<'static, u32, 32>,
    _vblank_interrupt: InterruptHandler<'static>,
}

struct Header {
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
                        if let Some(waker) = &task.join_waker {
                            waker.wake_by_ref();
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
            unsafe { &mut *ptr.cast::<MaybeUninit<TaskCell<F>>>().as_ptr() }.assume_init_drop();
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
    future: Pin<Rc<DebugRefCell<StoredHeader>>>,
}

struct StoredHeader {
    header: Header,
}

impl Deref for StoredHeader {
    type Target = Header;

    fn deref(&self) -> &Self::Target {
        &self.header
    }
}

impl DerefMut for StoredHeader {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.header
    }
}

impl Drop for StoredHeader {
    fn drop(&mut self) {
        let drop = self.header.vtable.drop;
        let p = NonNull::new(&mut **self as *mut Header).unwrap();

        unsafe { drop(p) };
    }
}

pub struct TaskJoinErased {
    future: Pin<Rc<DebugRefCell<StoredHeader>>>,
}

impl Future for TaskJoinErased {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let h = self.future.get();

        Header::poll(h.cast(), cx)
    }
}

pub struct TaskJoin<O> {
    future: Pin<Rc<DebugRefCell<StoredHeader>>>,
    _phantom: PhantomData<O>,
}

impl<O> Future for TaskJoin<O> {
    type Output = O;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let h = self.future.get();

        let mut ret = Poll::Pending;

        Header::try_read_value(h.cast(), &mut ret as *mut _ as *mut _, cx);

        ret
    }
}

impl<O> TaskJoin<O> {
    pub fn abort(self) {
        let h = self.future.get();

        Header::abort(h.cast());
    }

    #[must_use]
    pub fn is_done(&self) -> bool {
        let h = self.future.get();

        Header::is_done(h.cast())
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
                vtable: TaskVTable::new::<F>(),
            },
            task: TaskRun {
                future: Stage::Running(future),
                join_waker: None,
            },
        };

        let rc = Rc::new(DebugRefCell::new(task));

        let erased = unsafe {
            let ptr = Rc::into_raw(rc);
            Pin::new_unchecked(Rc::<DebugRefCell<StoredHeader>>::from_raw(ptr as *const _))
        };

        (
            Task {
                future: erased.clone(),
            },
            TaskJoin {
                future: erased,
                _phantom: PhantomData,
            },
        )
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        let h = self.future.get();

        Header::poll(h.cast(), context)
    }
}

impl Executor {
    unsafe fn new() -> Self {
        let (reader, _) = INTERRUPTS.get_rw_ref();

        Executor {
            futures: slotmap::SlotMap::new(),
            interrupt_reader: reader,
            to_poll: Rc::new(RefCell::new(ToPoll {
                poll: Vec::new(),
                vblank_waiting: Vec::new(),
            })),
            _vblank_interrupt: interrupt::add_interrupt_handler(Interrupt::VBlank, |_| {
                CURRENT_VBLANK.write(CURRENT_VBLANK.read() + 1);
            }),
        }
    }
}

#[derive(Clone)]
struct TaskWaker {
    id: DefaultKey,
    to_poll: Rc<RefCell<ToPoll>>,
}

impl TaskWaker {
    unsafe fn from_ptr(ptr: *const ()) -> Rc<TaskWaker> {
        unsafe { Rc::from_raw(ptr as *const TaskWaker) }
    }

    fn get_raw_waker(self: Rc<Self>) -> RawWaker {
        fn clone(data: *const ()) -> RawWaker {
            let me = unsafe { Rc::from_raw(data as *const TaskWaker) };

            core::mem::forget(Rc::clone(&me));
            me.get_raw_waker()
        }

        fn wake(data: *const ()) {
            let me = unsafe { Rc::from_raw(data as *const TaskWaker) };
            me.to_poll.borrow_mut().poll.push(me.id);
        }

        fn wake_by_ref(data: *const ()) {
            let me = unsafe { Rc::from_raw(data as *const TaskWaker) };
            me.to_poll.borrow_mut().poll.push(me.id);
            core::mem::forget(me);
        }

        fn drop(data: *const ()) {
            let me = unsafe { Rc::from_raw(data as *const TaskWaker) };
            core::mem::drop(me);
        }

        let vtable = &RawWakerVTable::new(clone, wake, wake_by_ref, drop);

        RawWaker::new(Rc::into_raw(self) as *const (), vtable)
    }

    fn get_waker(self) -> Waker {
        unsafe { Waker::from_raw(Rc::new(self).get_raw_waker()) }
    }
}

impl Executor {
    fn get_to_poll(&mut self) -> Option<Vec<DefaultKey>> {
        let mut poll = self.to_poll.borrow_mut();

        if poll.poll.is_empty() {
            return None;
        }

        let mut to_poll = Default::default();
        core::mem::swap(&mut poll.poll, &mut to_poll);

        Some(to_poll)
    }

    fn get_waker_for_task(&self, task: DefaultKey) -> Waker {
        TaskWaker {
            id: task,
            to_poll: Rc::clone(&self.to_poll),
        }
        .get_waker()
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.add_waiting();

            while let Some(to_poll) = self.get_to_poll() {
                for text_index in to_poll {
                    let waker = self.get_waker_for_task(text_index);

                    let task = &mut self.futures[text_index];

                    let mut ctx = Context::from_waker(&waker);
                    match task.poll(&mut ctx) {
                        Poll::Ready(()) => {
                            self.futures.remove(text_index);
                        }
                        Poll::Pending => {} // no work required
                    }
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
            let idx = self.futures.insert(task);
            self.to_poll.borrow_mut().poll.push(idx);
        }
    }

    fn process_interrupts(&mut self) {
        while let Some(interrupts) = self.interrupt_reader.try_read() {
            if interrupts & (1 << Interrupt::VBlank as u32) != 0 {
                let poll = self.to_poll.borrow_mut();
                let (mut poll, mut vblank) =
                    RefMut::map_split(poll, |f| (&mut f.poll, &mut f.vblank_waiting));
                poll.append(&mut *vblank);
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

pub struct Scope<'scope, 'env: 'scope> {
    joiners: RefCell<Vec<TaskJoinErased>>,
    scope: PhantomData<&'scope mut &'scope ()>,
    env: PhantomData<&'env mut &'env ()>,
}

impl<'scope, 'env> Scope<'scope, 'env> {
    pub fn spawn<F>(&'scope self, f: F) -> TaskJoin<F::Output>
    where
        F: Future,
    {
        let (task, join) = unsafe { Task::new(f) };

        let erased = TaskJoinErased {
            future: join.future.clone(),
        };

        crate::println!("Spawning task");

        TO_ADD_TO_EXECUTOR.cell.borrow_mut().push(task);

        self.joiners.borrow_mut().push(erased);

        join
    }
}

pub async fn scoped<'env, F, T>(f: F) -> T
where
    F: for<'scope> FnOnce(&'scope Scope<'scope, 'env>) -> T,
{
    let scope = Scope {
        joiners: RefCell::new(Vec::new()),
        scope: PhantomData,
        env: PhantomData,
    };
    let r = f(&scope);

    let joiners = scope.joiners.take();

    for join in joiners {
        join.await;
    }

    r
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
