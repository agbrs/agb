use std::{collections::HashMap, hash::BuildHasher};

use crate::{colour::Colour, image_loader::Image};

pub struct Transformation {
    pub vflip: bool,
    pub hflip: bool,
}

impl Transformation {
    pub fn none() -> Self {
        Self {
            vflip: false,
            hflip: false,
        }
    }

    pub fn vflipped() -> Self {
        Self {
            vflip: true,
            hflip: false,
        }
    }

    pub fn hflipped() -> Self {
        Self {
            vflip: false,
            hflip: true,
        }
    }

    pub fn vhflipped() -> Self {
        Self {
            vflip: true,
            hflip: true,
        }
    }
}

pub struct DeduplicatedData {
    pub new_index: usize,
    pub transformation: Transformation,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct Tile {
    data: [Colour; 64],
}

impl Tile {
    fn split_image(input: &Image) -> Vec<Self> {
        let mut ret = vec![];

        for y in 0..(input.height / 8) {
            for x in 0..(input.width / 8) {
                let mut tile_data = Vec::with_capacity(64);

                for j in 0..8 {
                    for i in 0..8 {
                        tile_data.push(input.colour(x * 8 + i, y * 8 + j));
                    }
                }

                ret.push(Tile {
                    data: tile_data.try_into().unwrap(),
                });
            }
        }

        ret
    }

    fn vflipped(&self) -> Self {
        let mut new_data = self.data;
        for y in 0..4 {
            for x in 0..8 {
                new_data.swap(y * 8 + x, (7 - y) * 8 + x);
            }
        }

        Self { data: new_data }
    }

    fn hflipped(&self) -> Self {
        let mut new_data = self.data;

        for y in 0..8 {
            for x in 0..4 {
                new_data.swap(y * 8 + x, y * 8 + (7 - x));
            }
        }

        Self { data: new_data }
    }
}

pub(crate) fn deduplicate_image(input: &Image, can_flip: bool) -> (Image, Vec<DeduplicatedData>) {
    let mut resulting_tiles = vec![];
    let mut deduplication_data = vec![];

    let all_tiles = Tile::split_image(input);
    let mut existing_tiles = HashMap::new();

    let hasher = std::collections::hash_map::RandomState::new();

    for tile in all_tiles {
        let (tile, transformation) = if can_flip {
            let vflipped = tile.vflipped();
            let hflipped = tile.hflipped();
            let vhflipped = vflipped.hflipped();

            // find the one with the smallest hash
            let tile_hash = hasher.hash_one(&tile);
            let vflipped_hash = hasher.hash_one(&vflipped);
            let hflipped_hash = hasher.hash_one(&hflipped);
            let vhflipped_hash = hasher.hash_one(&vhflipped);

            let minimum = tile_hash
                .min(vflipped_hash)
                .min(hflipped_hash)
                .min(vhflipped_hash);

            if minimum == tile_hash {
                (tile, Transformation::none())
            } else if minimum == vflipped_hash {
                (vflipped, Transformation::vflipped())
            } else if minimum == hflipped_hash {
                (hflipped, Transformation::hflipped())
            } else {
                (vhflipped, Transformation::vhflipped())
            }
        } else {
            (tile, Transformation::none())
        };

        let index = *existing_tiles.entry(tile.clone()).or_insert_with(|| {
            resulting_tiles.push(tile);
            resulting_tiles.len() - 1
        });

        deduplication_data.push(DeduplicatedData {
            new_index: index,
            transformation,
        });
    }

    let image_data = resulting_tiles
        .iter()
        .flat_map(|tile| tile.data)
        .collect::<Vec<_>>();
    (Image::from_colour_data(image_data), deduplication_data)
}
