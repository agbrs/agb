use core::cell::RefCell;

type Index = u8;

pub struct Loan<'a, const S: usize> {
    pub my_index: Index,
    arena: &'a RefCell<ArenaInner<S>>,
}

#[derive(Debug)]
struct ArenaInner<const S: usize> {
    arena: [Index; S],
    first: Index,
}

pub struct Arena<const S: usize> {
    arena_inner: RefCell<ArenaInner<S>>,
}

impl<const S: usize> Arena<S> {
    pub fn new() -> Arena<S> {
        // we use the special value u8::MAX as a None
        assert!(S < u8::MAX as usize - 1);

        let mut arena: [u8; S] = [u8::MAX; S];
        arena
            .iter_mut()
            .enumerate()
            .for_each(|(idx, a)| *a = idx as Index + 1);

        if let Some(a) = arena.last_mut() {
            *a = u8::MAX;
        }

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
        arena.first = arena.arena[i as usize];

        arena.arena[i as usize] = 1;
        Some(Loan {
            my_index: i,
            arena: &self.arena_inner,
        })
    }
}

impl<const S: usize> Drop for Loan<'_, S> {
    fn drop(&mut self) {
        let mut arena = self.arena.borrow_mut();
        let mut me = arena.arena[self.my_index as usize];

        me -= 1;
        if me == 0 {
            arena.arena[self.my_index as usize] = arena.first;
            arena.first = self.my_index;
        } else {
            arena.arena[self.my_index as usize] = me;
        }
    }
}

impl<const S: usize> Clone for Loan<'_, S> {
    fn clone(&self) -> Self {
        self.arena.borrow_mut().arena[self.my_index as usize] += 1;

        Loan {
            my_index: self.my_index,
            arena: self.arena,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test_case]
    fn get_everything(_gba: &mut crate::Gba) {
        let s: Arena<4> = Arena::new();
        {
            let c = alloc::vec![s.get_next_free(), s.get_next_free()];
            c.iter()
                .enumerate()
                .for_each(|(i, a)| assert!(a.is_some(), "expected index {} is some", i));
        }
        {
            let c = alloc::vec![
                s.get_next_free(),
                s.get_next_free(),
                s.get_next_free(),
                s.get_next_free()
            ];
            c.iter().for_each(|a| assert!(a.is_some()));
            assert!(s.get_next_free().is_none());
        }
        {
            let c = alloc::vec![
                s.get_next_free(),
                s.get_next_free(),
                s.get_next_free(),
                s.get_next_free()
            ];
            c.iter().for_each(|a| assert!(a.is_some()));
            assert!(s.get_next_free().is_none());
        }
    }
}
