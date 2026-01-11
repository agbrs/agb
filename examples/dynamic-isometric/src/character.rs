use agb::{
    display::{
        GraphicsFrame, Priority,
        object::{GraphicsMode, Object, Tag},
    },
    fixnum::{Num, Vector2D, num, vec2},
    input::ButtonController,
};

use crate::{
    isometric_render::{Map, TileType, world_to_gba_tile_smooth},
    sprites,
};

pub struct Character {
    tag: &'static Tag,
    // position is the current foot location in world space
    position: Vector2D<Num<i32, 12>>,
    target_position: Vector2D<Num<i32, 12>>,

    foot_offset: Vector2D<i32>,
    flipped: bool,
}

impl Character {
    pub fn new(tag: &'static Tag, position: Vector2D<Num<i32, 12>>) -> Self {
        Self {
            tag,
            position,
            target_position: position,
            foot_offset: vec2(16, 30),
            flipped: false,
        }
    }

    pub fn update(&mut self, input: &ButtonController, wall_map: &Map, floor_map: &Map) {
        let just_pressed = input.just_pressed_vector::<Num<i32, 12>>();
        if just_pressed != vec2(num!(0), num!(0)) {
            if self.target_position != self.position {
                self.position = self.target_position;
            }

            self.flipped = just_pressed.x > num!(0) || just_pressed.y < num!(0);

            let new_location = self.target_position + just_pressed;
            if wall_map.get_tile(new_location.floor()) == TileType::Air
                && floor_map.get_tile(new_location.floor()) != TileType::Air
            {
                self.target_position = new_location;
            }
        }

        self.position = (self.position + self.target_position) / 2;
    }

    pub fn show(&self, frame: &mut GraphicsFrame, wall_map: &Map) {
        // which priority do we need for the bottom sprites?
        let tile_pos = self.position.round();
        let priority = if wall_map.get_tile(tile_pos + vec2(1, 0)) != TileType::Air
            || wall_map.get_tile(tile_pos + vec2(1, 1)) != TileType::Air
            || wall_map.get_tile(tile_pos + vec2(0, 1)) != TileType::Air
        {
            Priority::P3
        } else {
            Priority::P1
        };

        let real_tile_space = world_to_gba_tile_smooth(self.position);
        let real_pixel_space = (real_tile_space * 8).round();

        Object::new(self.tag.sprite(0))
            .set_pos(real_pixel_space - self.foot_offset)
            .set_priority(Priority::P1)
            .set_hflip(self.flipped)
            .show(frame);
        Object::new(self.tag.sprite(1))
            .set_pos(real_pixel_space - self.foot_offset + vec2(0, 16))
            .set_priority(priority)
            .set_hflip(self.flipped)
            .show(frame);

        // drop shadow
        Object::new(sprites::DROP_SHADOW.sprite(0))
            .set_pos(real_pixel_space - vec2(16, 8))
            .set_priority(priority)
            .set_graphics_mode(GraphicsMode::AlphaBlending)
            .show(frame);
    }
}
