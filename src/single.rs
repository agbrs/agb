use core::cell::Cell;

pub struct Singleton<T> {
    single: Option<T>,
}

impl<T> Singleton<T> {
    pub const fn new(s: T) -> Self {
        Singleton { single: Some(s) }
    }
    pub fn take(&mut self) -> T {
        let g = core::mem::replace(&mut self.single, None);
        g.unwrap()
    }
}

pub struct Single {
    is_taken: Cell<bool>,
}

pub struct SingleToken<'a> {
    cell: &'a Cell<bool>,
}

impl Single {
    pub const fn new() -> Self {
        Single {
            is_taken: Cell::new(false),
        }
    }

    pub fn take(&self) -> Result<SingleToken, &'static str> {
        if self.is_taken.get() {
            Err("Already taken")
        } else {
            self.is_taken.set(true);
            Ok(SingleToken {
                cell: &self.is_taken,
            })
        }
    }
}

impl Drop for SingleToken<'_> {
    fn drop(&mut self) {
        self.cell.set(false);
    }
}
