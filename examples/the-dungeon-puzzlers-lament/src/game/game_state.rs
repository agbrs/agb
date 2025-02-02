use agb::{
    display::{
        object::{OamFrame, Object, Tag},
        tiled::RegularBackgroundTiles,
    },
    fixnum::Vector2D,
    input::{Button, ButtonController, Tri},
};
use alloc::{vec, vec::Vec};

use crate::{
    level::{Item, Level},
    map::MapElement,
    resources,
    sfx::Sfx,
};

use super::simulation::{Direction, Simulation};

pub const PLAY_AREA_WIDTH: usize = 11;
pub const PLAY_AREA_HEIGHT: usize = 10;

const ITEM_AREA_WIDTH: usize = 3;
const ITEM_AREA_HEIGHT: usize = 3;

const ITEM_AREA_TOP_LEFT: Vector2D<i32> = Vector2D::new(179, 96);
const CURSOR_OFFSET: Vector2D<i32> = Vector2D::new(14, 14);

const ARROW_TOP_LEFT: Vector2D<i32> = Vector2D::new(175, 15);

pub struct GameState {
    level_number: usize,
    level: &'static Level,
    cursor_state: CursorState,
    frame: usize,

    item_states: Vec<ItemState>,
}

impl GameState {
    pub fn new(level_number: usize) -> Self {
        let level = Level::get_level(level_number);

        let position = level
            .entities
            .iter()
            .find(|x| x.0 == Item::Hero)
            .map(|hero| hero.1.x as usize + PLAY_AREA_WIDTH * hero.1.y as usize)
            .unwrap_or(PLAY_AREA_WIDTH * PLAY_AREA_HEIGHT / 2 + PLAY_AREA_WIDTH / 2);

        Self {
            level_number,
            level,
            cursor_state: CursorState {
                item_position: 0,
                board_position: position,
                current_place: CursorPlace::Item,
                held_item: None,
            },
            frame: 0,

            item_states: vec![ItemState::default(); level.items.len()],
        }
    }

    pub fn create_simulation(&self, sfx: &mut Sfx) -> Simulation {
        Simulation::generate(
            self.item_states
                .iter()
                .zip(self.level.items)
                .filter_map(|(location, item)| match location {
                    ItemState::Placed(loc) => Some((*loc, *item)),
                    ItemState::NotPlaced => None,
                })
                .map(|(location, item)| {
                    (
                        item,
                        Vector2D::new(
                            (location % PLAY_AREA_WIDTH) as i32,
                            (location / PLAY_AREA_WIDTH) as i32,
                        ),
                    )
                })
                .chain(self.level.entities.iter().map(|x| (x.0, x.1))),
            self.level,
            sfx,
        )
    }

    pub fn load_level_background(&self, map: &mut RegularBackgroundTiles) {
        crate::backgrounds::load_level_background(map, self.level_number);
    }

    pub fn force_place(&mut self) {
        if self.cursor_state.current_place == CursorPlace::Board {
            let position_x = (self.cursor_state.board_position % PLAY_AREA_WIDTH) as i32;
            let position_y = (self.cursor_state.board_position / PLAY_AREA_WIDTH) as i32;

            let position: Vector2D<_> = (position_x, position_y).into();

            let map_tile = self.level.map[(position_x, position_y)];

            let fixed_item_at_location = self
                .level
                .entities
                .iter()
                .any(|entity| entity.1 == position);

            if map_tile == MapElement::Floor && !fixed_item_at_location {
                let placeable_item_at_location =
                    self.item_states.iter().position(|state| match state {
                        ItemState::Placed(position) => {
                            *position == self.cursor_state.board_position
                        }
                        ItemState::NotPlaced => false,
                    });

                if placeable_item_at_location.is_none() {
                    if let Some(held_item) = self.cursor_state.held_item {
                        self.item_states[held_item] =
                            ItemState::Placed(self.cursor_state.board_position);
                        self.cursor_state.held_item = None;
                    }
                }
            }
        }
    }

    pub fn step(&mut self, input: &ButtonController, sfx: &mut Sfx) {
        self.frame = self.frame.wrapping_add(1);

        self.cursor_state.update_position(input);

        if input.is_just_pressed(Button::A) {
            match self.cursor_state.current_place {
                CursorPlace::Board => {
                    let position_x = (self.cursor_state.board_position % PLAY_AREA_WIDTH) as i32;
                    let position_y = (self.cursor_state.board_position / PLAY_AREA_WIDTH) as i32;

                    let position: Vector2D<_> = (position_x, position_y).into();

                    let map_tile = self.level.map[(position_x, position_y)];

                    let fixed_item_at_location = self
                        .level
                        .entities
                        .iter()
                        .any(|entity| entity.1 == position);

                    if map_tile == MapElement::Floor && !fixed_item_at_location {
                        let placeable_item_at_location =
                            self.item_states.iter().position(|state| match state {
                                ItemState::Placed(position) => {
                                    *position == self.cursor_state.board_position
                                }
                                ItemState::NotPlaced => false,
                            });

                        let played_sound = if let Some(held_item) = self.cursor_state.held_item {
                            self.item_states[held_item] =
                                ItemState::Placed(self.cursor_state.board_position);
                            self.cursor_state.held_item = None;

                            sfx.place();
                            true
                        } else {
                            false
                        };

                        if let Some(placeable_item_at_location) = placeable_item_at_location {
                            self.cursor_state.held_item = Some(placeable_item_at_location);
                            self.item_states[placeable_item_at_location] = ItemState::NotPlaced;

                            if !played_sound {
                                sfx.select();
                            }
                        }
                    } else {
                        sfx.bad_selection();
                    }
                }
                CursorPlace::Item => {
                    let item_position = self.cursor_state.item_position;

                    if matches!(
                        self.item_states.get(item_position),
                        Some(ItemState::NotPlaced)
                    ) {
                        sfx.select();
                        self.cursor_state.current_place = CursorPlace::Board;
                        self.cursor_state.held_item = Some(item_position);
                    } else {
                        sfx.bad_selection();
                    }
                }
            }
        }

        if input.is_just_pressed(Button::B) {
            match self.cursor_state.current_place {
                CursorPlace::Item => {
                    self.cursor_state.current_place = CursorPlace::Board;
                    self.cursor_state.held_item = None;
                }
                CursorPlace::Board => {
                    self.cursor_state.current_place = CursorPlace::Item;
                    self.cursor_state.held_item = None;
                }
            }
        }
    }

    pub fn render_arrows(&self, oam: &mut OamFrame, current_turn: Option<usize>) {
        let is_odd_frame = if current_turn.is_some() {
            true
        } else {
            let frame_index = self.frame / 32;
            frame_index % 2 == 1
        };

        for (i, direction) in self.level.directions.iter().enumerate() {
            let x = (i % 4) as i32;
            let y = (i / 4) as i32;

            let arrow_position = ARROW_TOP_LEFT + (x * 15, y * 15).into();
            let arrow_position = if is_odd_frame {
                arrow_odd_frame_offset(*direction)
            } else {
                (0, 0).into()
            } + arrow_position;

            let sprite_idx = if Some(i) == current_turn { 1 } else { 0 };

            let mut arrow_obj = Object::new(arrow_for_direction(*direction).sprite(sprite_idx));
            arrow_obj.set_position(arrow_position);

            oam.set(&arrow_obj);
        }
    }

    pub fn render(&self, oam: &mut OamFrame) {
        let frame_index = self.frame / 32;
        let is_odd_frame = frame_index % 2 == 1;

        let mut cursor_obj = Object::new(resources::CURSOR.sprite(0));
        cursor_obj.set_position(self.cursor_state.get_position(is_odd_frame));

        oam.set(&cursor_obj);

        let level = self.level;

        self.render_arrows(oam, None);

        fn placed_position(position: usize, item: &Item) -> Vector2D<i32> {
            let position_x = (position % PLAY_AREA_WIDTH) as i32;
            let position_y = (position / PLAY_AREA_WIDTH) as i32;
            let position = Vector2D::new(position_x, position_y);

            position * 16 + item.map_entity_offset()
        }

        if let Some(held) = self.cursor_state.held_item {
            let item = &level.items[held];
            let item_position = placed_position(self.cursor_state.board_position, item);
            let mut item_obj = Object::new(item.tag().animation_sprite(frame_index));
            item_obj.set_position(item_position);

            oam.set(&item_obj);
        }

        for (item_position, item) in level.items.iter().enumerate().filter_map(|(i, item)| {
            let item_position = match self.item_states[i] {
                ItemState::Placed(position) => placed_position(position, item),
                ItemState::NotPlaced => {
                    if self.cursor_state.held_item == Some(i) {
                        return None;
                    } else {
                        let x = (i % ITEM_AREA_WIDTH) as i32;
                        let y = (i / ITEM_AREA_WIDTH) as i32;

                        ITEM_AREA_TOP_LEFT + (x * 16, y * 16).into()
                    }
                }
            };

            Some((item_position, item))
        }) {
            let mut item_obj = Object::new(item.tag().animation_sprite(frame_index));
            item_obj.set_position(item_position);

            oam.set(&item_obj);
        }

        for entity in level.entities.iter() {
            let entity_position = entity.1 * 16 + entity.0.map_entity_offset();

            let mut entity_obj = Object::new(entity.0.shadow_tag().sprite(0));
            entity_obj.set_position(entity_position);

            oam.set(&entity_obj);
        }
    }
}

struct CursorState {
    item_position: usize,
    board_position: usize,
    current_place: CursorPlace,
    held_item: Option<usize>,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum CursorPlace {
    Item,
    Board,
}

impl CursorState {
    fn get_position(&self, is_odd_frame: bool) -> Vector2D<i32> {
        let odd_frame_offset = if is_odd_frame {
            Vector2D::new(1, 1)
        } else {
            Vector2D::new(0, 0)
        };
        let place_position: Vector2D<_> = match self.current_place {
            CursorPlace::Board => {
                let current_x = (self.board_position % PLAY_AREA_WIDTH) as i32;
                let current_y = (self.board_position / PLAY_AREA_WIDTH) as i32;

                (current_x * 16, current_y * 16).into()
            }
            CursorPlace::Item => {
                let current_x = (self.item_position % ITEM_AREA_WIDTH) as i32;
                let current_y = (self.item_position / ITEM_AREA_WIDTH) as i32;

                ITEM_AREA_TOP_LEFT + (current_x * 16, current_y * 16).into()
            }
        };

        place_position + CURSOR_OFFSET + odd_frame_offset
    }

    fn update_position(&mut self, input: &ButtonController) {
        let ud: Tri = (
            input.is_just_pressed(Button::UP),
            input.is_just_pressed(Button::DOWN),
        )
            .into();
        let lr: Tri = (
            input.is_just_pressed(Button::LEFT),
            input.is_just_pressed(Button::RIGHT),
        )
            .into();

        if ud == Tri::Zero && lr == Tri::Zero {
            return;
        }

        match self.current_place {
            CursorPlace::Board => {
                let current_x = self.board_position % PLAY_AREA_WIDTH;
                let current_y = self.board_position / PLAY_AREA_WIDTH;

                let mut new_x = current_x.saturating_add_signed(lr as isize).max(1);
                let new_y = current_y
                    .saturating_add_signed(ud as isize)
                    .clamp(1, PLAY_AREA_HEIGHT - 2);

                if new_x == PLAY_AREA_WIDTH - 1 {
                    new_x = new_x.min(PLAY_AREA_WIDTH - 2);

                    if self.held_item.is_none() {
                        self.current_place = CursorPlace::Item;
                        self.item_position = new_y.saturating_sub(5).clamp(0, ITEM_AREA_HEIGHT - 1)
                            * ITEM_AREA_WIDTH;
                    }
                }

                self.board_position = new_x + new_y * PLAY_AREA_WIDTH;
            }
            CursorPlace::Item => {
                let current_x = self.item_position % ITEM_AREA_WIDTH;
                let current_y = self.item_position / ITEM_AREA_WIDTH;

                let mut new_x = current_x.wrapping_add_signed(lr as isize);
                let new_y = current_y
                    .saturating_add_signed(ud as isize)
                    .min(ITEM_AREA_HEIGHT - 1);

                if new_x == usize::MAX {
                    new_x = 0;

                    self.current_place = CursorPlace::Board;
                    self.board_position = (new_y + 5) * PLAY_AREA_WIDTH + PLAY_AREA_WIDTH - 2;
                } else {
                    new_x = new_x.min(ITEM_AREA_WIDTH - 1);
                }

                self.item_position = new_x + new_y * ITEM_AREA_WIDTH;
            }
        }
    }
}

fn arrow_for_direction(direction: Direction) -> &'static Tag {
    match direction {
        Direction::Up => resources::ARROW_UP,
        Direction::Down => resources::ARROW_DOWN,
        Direction::Left => resources::ARROW_LEFT,
        Direction::Right => resources::ARROW_RIGHT,
    }
}

const fn arrow_odd_frame_offset(direction: Direction) -> Vector2D<i32> {
    match direction {
        Direction::Up => Vector2D::new(0, -1),
        Direction::Down => Vector2D::new(0, 1),
        Direction::Left => Vector2D::new(-1, 0),
        Direction::Right => Vector2D::new(1, 0),
    }
}

#[derive(Default, Clone)]
enum ItemState {
    Placed(usize),
    #[default]
    NotPlaced,
}
