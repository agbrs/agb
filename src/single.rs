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
    is_taken: bool,
}

pub struct SingleToken<'a> {
    cell: &'a mut bool,
}

impl Single {
    pub const fn new() -> Self {
        Single { is_taken: false }
    }

    pub fn take(&mut self) -> Result<SingleToken, &'static str> {
        if self.is_taken {
            Err("Already taken")
        } else {
            self.is_taken = true;
            Ok(SingleToken {
                cell: &mut self.is_taken,
            })
        }
    }
}

impl Drop for SingleToken<'_> {
    fn drop(&mut self) {
        (*self.cell) = false;
    }
}
