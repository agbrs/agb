// These benchmarks were taken from hashbrown. They are impossible to run
// on the target GBA hardware, but hopefully running these on something like a
// raspberry pi zero will give something comparable.

// This benchmark suite contains some benchmarks along a set of dimensions:
//   Int key distribution: low bit heavy, top bit heavy, and random.
//   Task: basic functionality: insert, insert_erase, lookup, lookup_fail, iter
#![feature(test)]

extern crate test;

use test::{Bencher, black_box};

use agb_hashmap::HashMap;
use std::sync::atomic::{self, AtomicUsize};

const SIZE: usize = 1000;

type StdHashMap<K, V> = std::collections::hash_map::HashMap<K, V>;

// A random key iterator.
#[derive(Clone, Copy)]
struct RandomKeys {
    state: usize,
}

impl RandomKeys {
    fn new() -> Self {
        RandomKeys { state: 0 }
    }
}

impl Iterator for RandomKeys {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        // Add 1 then multiply by some 32 bit prime.
        self.state = self.state.wrapping_add(1).wrapping_mul(3_787_392_781);
        Some(self.state)
    }
}

// Just an arbitrary side effect to make the maps not shortcircuit to the non-dropping path
// when dropping maps/entries (most real world usages likely have drop in the key or value)
lazy_static::lazy_static! {
    static ref SIDE_EFFECT: AtomicUsize = AtomicUsize::new(0);
}

#[derive(Clone)]
struct DropType(usize);
impl Drop for DropType {
    fn drop(&mut self) {
        SIDE_EFFECT.fetch_add(self.0, atomic::Ordering::SeqCst);
    }
}

macro_rules! bench_suite {
    ($bench_macro:ident, $bench_agb_hashmap_serial:ident, $bench_std_serial:ident,
     $bench_agb_hashmap_highbits:ident, $bench_std_highbits:ident,
     $bench_agb_hashmap_random:ident, $bench_std_random:ident) => {
        $bench_macro!($bench_agb_hashmap_serial, HashMap, 0..);
        $bench_macro!($bench_std_serial, StdHashMap, 0..);
        $bench_macro!(
            $bench_agb_hashmap_highbits,
            HashMap,
            (0..).map(usize::swap_bytes)
        );
        $bench_macro!(
            $bench_std_highbits,
            StdHashMap,
            (0..).map(usize::swap_bytes)
        );
        $bench_macro!($bench_agb_hashmap_random, HashMap, RandomKeys::new());
        $bench_macro!($bench_std_random, StdHashMap, RandomKeys::new());
    };
}

macro_rules! bench_insert {
    ($name:ident, $maptype:ident, $keydist:expr) => {
        #[bench]
        fn $name(b: &mut Bencher) {
            let mut m = $maptype::with_capacity(SIZE);
            b.iter(|| {
                m.clear();
                for i in ($keydist).take(SIZE) {
                    m.insert(i, (DropType(i), [i; 20]));
                }
                black_box(&mut m);
            });
            eprintln!("{}", SIDE_EFFECT.load(atomic::Ordering::SeqCst));
        }
    };
}

bench_suite!(
    bench_insert,
    agb_hashmap_insert_serial,
    std_hashmap_insert_serial,
    agb_hashmap_insert_highbits,
    std_hashmap_insert_highbits,
    agb_hashmap_insert_random,
    std_hashmap_insert_random
);

macro_rules! bench_grow_insert {
    ($name:ident, $maptype:ident, $keydist:expr) => {
        #[bench]
        fn $name(b: &mut Bencher) {
            b.iter(|| {
                let mut m = $maptype::default();
                for i in ($keydist).take(SIZE) {
                    m.insert(i, DropType(i));
                }
                black_box(&mut m);
            })
        }
    };
}

bench_suite!(
    bench_grow_insert,
    agb_hashmap_grow_insert_serial,
    std_hashmap_grow_insert_serial,
    agb_hashmap_grow_insert_highbits,
    std_hashmap_grow_insert_highbits,
    agb_hashmap_grow_insert_random,
    std_hashmap_grow_insert_random
);

macro_rules! bench_insert_erase {
    ($name:ident, $maptype:ident, $keydist:expr) => {
        #[bench]
        fn $name(b: &mut Bencher) {
            let mut base = $maptype::default();
            for i in ($keydist).take(SIZE) {
                base.insert(i, DropType(i));
            }
            let skip = $keydist.skip(SIZE);
            b.iter(|| {
                let mut m = base.clone();
                let mut add_iter = skip.clone();
                let mut remove_iter = $keydist;
                // While keeping the size constant,
                // replace the first keydist with the second.
                for (add, remove) in (&mut add_iter).zip(&mut remove_iter).take(SIZE) {
                    m.insert(add, DropType(add));
                    black_box(m.remove(&remove));
                }
                black_box(m);
            });
            eprintln!("{}", SIDE_EFFECT.load(atomic::Ordering::SeqCst));
        }
    };
}

bench_suite!(
    bench_insert_erase,
    agb_hashmap_insert_erase_serial,
    std_hashmap_insert_erase_serial,
    agb_hashmap_insert_erase_highbits,
    std_hashmap_insert_erase_highbits,
    agb_hashmap_insert_erase_random,
    std_hashmap_insert_erase_random
);

macro_rules! bench_lookup {
    ($name:ident, $maptype:ident, $keydist:expr) => {
        #[bench]
        fn $name(b: &mut Bencher) {
            let mut m = $maptype::default();
            for i in $keydist.take(SIZE) {
                m.insert(i, DropType(i));
            }

            b.iter(|| {
                for i in $keydist.take(SIZE) {
                    black_box(m.get(&i));
                }
            });
            eprintln!("{}", SIDE_EFFECT.load(atomic::Ordering::SeqCst));
        }
    };
}

bench_suite!(
    bench_lookup,
    agb_hashmap_lookup_serial,
    std_hashmap_lookup_serial,
    agb_hashmap_lookup_highbits,
    std_hashmap_lookup_highbits,
    agb_hashmap_lookup_random,
    std_hashmap_lookup_random
);

macro_rules! bench_lookup_fail {
    ($name:ident, $maptype:ident, $keydist:expr) => {
        #[bench]
        fn $name(b: &mut Bencher) {
            let mut m = $maptype::default();
            let mut iter = $keydist;
            for i in (&mut iter).take(SIZE) {
                m.insert(i, DropType(i));
            }

            b.iter(|| {
                for i in (&mut iter).take(SIZE) {
                    black_box(m.get(&i));
                }
            })
        }
    };
}

bench_suite!(
    bench_lookup_fail,
    agb_hashmap_lookup_fail_serial,
    std_hashmap_lookup_fail_serial,
    agb_hashmap_lookup_fail_highbits,
    std_hashmap_lookup_fail_highbits,
    agb_hashmap_lookup_fail_random,
    std_hashmap_lookup_fail_random
);

macro_rules! bench_iter {
    ($name:ident, $maptype:ident, $keydist:expr) => {
        #[bench]
        fn $name(b: &mut Bencher) {
            let mut m = $maptype::default();
            for i in ($keydist).take(SIZE) {
                m.insert(i, DropType(i));
            }

            b.iter(|| {
                for i in &m {
                    black_box(i);
                }
            })
        }
    };
}

bench_suite!(
    bench_iter,
    agb_hashmap_iter_serial,
    std_hashmap_iter_serial,
    agb_hashmap_iter_highbits,
    std_hashmap_iter_highbits,
    agb_hashmap_iter_random,
    std_hashmap_iter_random
);

macro_rules! clone_bench {
    ($maptype:ident) => {
        use super::DropType;
        use test::{Bencher, black_box};

        #[bench]
        fn clone_small(b: &mut Bencher) {
            let mut m = $maptype::new();
            for i in 0..10 {
                m.insert(i, DropType(i));
            }

            b.iter(|| {
                black_box(m.clone());
            })
        }

        #[bench]
        fn clone_from_small(b: &mut Bencher) {
            let mut m = $maptype::new();
            let mut m2 = $maptype::new();
            for i in 0..10 {
                m.insert(i, DropType(i));
            }

            b.iter(|| {
                m2.clone_from(&m);
                black_box(&mut m2);
            })
        }

        #[bench]
        fn clone_large(b: &mut Bencher) {
            let mut m = $maptype::new();
            for i in 0..1000 {
                m.insert(i, DropType(i));
            }

            b.iter(|| {
                black_box(m.clone());
            })
        }

        #[bench]
        fn clone_from_large(b: &mut Bencher) {
            let mut m = $maptype::new();
            let mut m2 = $maptype::new();
            for i in 0..1000 {
                m.insert(i, DropType(i));
            }

            b.iter(|| {
                m2.clone_from(&m);
                black_box(&mut m2);
            })
        }
    };
}

mod agb_hashmap_clone_benches {
    use agb_hashmap::HashMap;
    clone_bench!(HashMap);
}

mod std_hashmap_clone_benches {
    use std::collections::hash_map::HashMap;
    clone_bench!(HashMap);
}
