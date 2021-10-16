use core::{cell::RefCell, num::NonZeroU8};

type Index = u8;

pub struct Loan<'a, const S: usize> {
    pub my_index: Index,
    arena: &'a RefCell<ArenaInner<S>>,
}

/// Next should be an option, however this raises this struct to 3 bytes where
/// it is now only 2 bytes This saves a byte per entry and therefore it is
/// probably worth it.
#[derive(Clone, Copy, Debug)]
enum Element {
    Contains(NonZeroU8),
    Next(Index),
}

#[derive(Debug)]
struct ArenaInner<const S: usize> {
    arena: [Element; S],
    first: Index,
}

pub struct Arena<const S: usize> {
    arena_inner: RefCell<ArenaInner<S>>,
}

impl<const S: usize> Arena<S> {
    pub fn new() -> Arena<S> {
        // we use the special value u8::MAX as a None
        assert!(S < u8::MAX as usize - 1);

        let mut arena: [Element; S] = [Element::Next(u8::MAX); S];
        arena
            .iter_mut()
            .enumerate()
            .for_each(|(index, e)| *e = Element::Next(index as Index + 1));
        arena.last_mut().map(|a| *a = Element::Next(u8::MAX));
        Arena {
            arena_inner: RefCell::new(ArenaInner { arena, first: 0 }),
        }
    }

    pub fn get_next_free(&self) -> Option<Loan<S>> {
        let mut arena = self.arena_inner.borrow_mut();
        let i = arena.first;
        if i == u8::MAX {
            return None;
        }
        if let Element::Next(n) = arena.arena[i as usize] {
            arena.first = n;
        } else {
            unreachable!("invalid state, next points to already occupied state");
        }
        arena.arena[i as usize] = Element::Contains(NonZeroU8::new(1).unwrap());
        Some(Loan {
            my_index: i,
            arena: &self.arena_inner,
        })
    }
}

impl<const S: usize> Drop for Loan<'_, S> {
    fn drop(&mut self) {
        let mut arena = self.arena.borrow_mut();
        let me = &mut arena.arena[self.my_index as usize];
        match me {
            Element::Contains(n) => {
                let mut a = n.get();
                a -= 1;
                if a == 0 {
                    arena.arena[self.my_index as usize] = Element::Next(arena.first);
                    arena.first = self.my_index;
                } else {
                    *n = NonZeroU8::new(a).unwrap();
                }
            }
            _ => unreachable!("if a loan exists the correspoinding arena entry should be filled"),
        }
    }
}

impl<const S: usize> Clone for Loan<'_, S> {
    fn clone(&self) -> Self {
        match &mut self.arena.borrow_mut().arena[self.my_index as usize] {
            Element::Contains(n) => {
                let a = n.get();
                *n = NonZeroU8::new(a + 1).unwrap();
            }
            _ => unreachable!("if a loan exists the correspoinding arena entry should be filled"),
        }
        Loan {
            my_index: self.my_index,
            arena: self.arena,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use alloc;

    #[test_case]
    fn size_of_element(_gba: &mut crate::Gba) {
        let s = core::mem::size_of::<Element>();
        assert_eq!(s, 2, "elements should be of a minimum size");
    }

    #[test_case]
    fn get_everything(_gba: &mut crate::Gba) {
        let s: Arena<4> = Arena::new();
        {
            let mut c = alloc::vec::Vec::new();
            c.push(s.get_next_free());
            c.push(s.get_next_free());
            c.iter().for_each(|a| assert!(a.is_some()));
        }
        {
            let mut c = alloc::vec::Vec::new();
            c.push(s.get_next_free());
            c.push(s.get_next_free());
            c.push(s.get_next_free());
            c.push(s.get_next_free());
            c.iter().for_each(|a| assert!(a.is_some()));
            assert!(s.get_next_free().is_none());
        }
        {
            let mut c = alloc::vec::Vec::new();
            c.push(s.get_next_free());
            c.push(s.get_next_free());
            c.push(s.get_next_free());
            c.push(s.get_next_free());
            c.iter().for_each(|a| assert!(a.is_some()));
            assert!(s.get_next_free().is_none());
        }
    }
}
