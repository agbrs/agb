pub struct Singleton<T> {
    single: Option<T>,
}

#[allow(dead_code)]
impl<T> Singleton<T> {
    pub const fn new(s: T) -> Self {
        Singleton { single: Some(s) }
    }
    pub const fn empty() -> Self {
        Singleton { single: None }
    }
    pub fn take(&mut self) -> T {
        let g = core::mem::replace(&mut self.single, None);
        g.unwrap()
    }
}
