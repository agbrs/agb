use alloc::vec::Vec;

pub struct Arena<T> {
    first: Option<usize>,
    data: Vec<Item<T>>,
}
enum Item<T> {
    Next {
        next: Option<usize>,
        generation: usize,
    },
    Item {
        item: T,
        generation: usize,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct Index {
    generation: usize,
    index: usize,
}

impl<T> Arena<T> {
    #[must_use]
    pub fn new() -> Self {
        Arena {
            first: None,
            data: Vec::new(),
        }
    }
    pub fn remove(&mut self, idx: Index) {
        if let Item::Item {
            item: _,
            generation,
        } = self.data[idx.index]
        {
            if generation != idx.generation {
                return;
            }

            self.data[idx.index] = Item::Next {
                next: self.first,
                generation,
            };
            self.first = Some(idx.index);
        }
    }
    pub fn get_mut(&mut self, idx: Index) -> Option<&mut T> {
        match &mut self.data[idx.index] {
            Item::Next {
                next: _,
                generation: _,
            } => None,
            Item::Item { item, generation } => {
                if *generation == idx.generation {
                    Some(item)
                } else {
                    None
                }
            }
        }
    }
    pub fn insert(&mut self, data: T) -> Index {
        match self.first {
            Some(idx) => {
                let (next, generation) = match &self.data[idx] {
                    Item::Next { next, generation } => (*next, *generation),
                    _ => unreachable!(),
                };
                self.data[idx] = Item::Item {
                    item: data,
                    generation: generation + 1,
                };

                self.first = next;
                Index {
                    generation: generation + 1,
                    index: idx,
                }
            }
            None => {
                self.data.push(Item::Item {
                    item: data,
                    generation: 0,
                });
                Index {
                    generation: 0,
                    index: self.data.len() - 1,
                }
            }
        }
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self::new()
    }
}
