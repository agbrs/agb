use core::ops::Index;

use agb::fixnum::Vector2D;

#[derive(Debug)]
pub struct Map<'map> {
    width: usize,
    height: usize,
    data: &'map [u8],
}

impl<'map> Map<'map> {
    pub const fn new(width: usize, height: usize, data: &'map [u8]) -> Self {
        assert!((width * height).div_ceil(8) == data.len());
        Self {
            width,
            height,
            data,
        }
    }

    pub const fn get(&self, index: Vector2D<i32>) -> MapElement {
        let (x, y) = (index.x, index.y);

        if x > self.width as i32 || x < 0 || y > self.height as i32 || y < 0 {
            MapElement::Wall
        } else {
            let position = x as usize + y as usize * self.width;
            let index = position / 8;
            let bit = position % 8;
            if (self.data[index] & (1 << bit)) != 0 {
                MapElement::Wall
            } else {
                MapElement::Floor
            }
        }
    }
}

impl Index<(i32, i32)> for Map<'_> {
    type Output = MapElement;

    fn index(&self, index: (i32, i32)) -> &Self::Output {
        &self[Into::<Vector2D<i32>>::into(index)]
    }
}

impl Index<Vector2D<i32>> for Map<'_> {
    type Output = MapElement;

    fn index(&self, index: Vector2D<i32>) -> &Self::Output {
        const WALL: MapElement = MapElement::Wall;
        const FLOOR: MapElement = MapElement::Floor;

        match self.get(index) {
            MapElement::Wall => &WALL,
            MapElement::Floor => &FLOOR,
        }
    }
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Copy)]
pub enum MapElement {
    #[default]
    Wall,
    Floor,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn check_default_is_correct(_: &mut agb::Gba) {
        let map = Map {
            width: 0,
            height: 0,
            data: &[],
        };

        assert_eq!(map[(-1, -1)], MapElement::Wall);
    }
}
