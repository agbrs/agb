use agb::{display::object::Tag, fixnum::Vector2D};

use crate::{game::Direction, map::Map, resources};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Item {
    Sword,
    Slime,
    Hero,
    Stairs,
    Door,
    Key,
    SwitchedOpenDoor,
    SwitchedClosedDoor,
    Switch,
    SwitchPressed,
    SpikesUp,
    SpikesDown,
    SquidUp,
    SquidDown,
    Ice,
    MovableBlock,
    Glove,
    Teleporter,
}

impl Item {
    pub fn shadow_tag(&self) -> &'static Tag {
        match self {
            Item::Sword => resources::SWORD_SHADOW,
            Item::Slime => resources::SLIME_SHADOW,
            Item::Hero => resources::HERO,
            Item::Stairs => resources::STAIRS,
            Item::Door => resources::DOOR,
            Item::Key => resources::KEY_SHADOW,
            Item::SwitchedOpenDoor => resources::SWITCHED_DOOR_OPEN,
            Item::SwitchedClosedDoor => resources::SWITCHED_DOOR_CLOSED,
            Item::Switch => resources::BUTTON_OFF,
            Item::SwitchPressed => resources::BUTTON_ON,
            Item::SpikesUp => resources::SPIKES_ON,
            Item::SpikesDown => resources::SPIKES_OFF,
            Item::SquidUp => resources::SQUID_UP_SHADOW,
            Item::SquidDown => resources::SQUID_DOWN_SHADOW,
            Item::Ice => resources::ICE,
            Item::MovableBlock => resources::ROCK_SHADOW,
            Item::Glove => resources::POW_GLOVE_SHADOW,
            Item::Teleporter => resources::TELEPORTER,
        }
    }

    pub fn tag(&self) -> &'static Tag {
        match self {
            Item::Sword => resources::SWORD,
            Item::Slime => resources::SLIME,
            Item::Hero => resources::HERO,
            Item::Stairs => resources::STAIRS,
            Item::Door => resources::DOOR,
            Item::Key => resources::KEY,
            Item::SwitchedOpenDoor => resources::SWITCHED_DOOR_OPEN,
            Item::SwitchedClosedDoor => resources::SWITCHED_DOOR_CLOSED,
            Item::Switch => resources::BUTTON_OFF,
            Item::SwitchPressed => resources::BUTTON_ON,
            Item::SpikesUp => resources::SPIKES_ON,
            Item::SpikesDown => resources::SPIKES_OFF,
            Item::SquidUp => resources::SQUID_UP,
            Item::SquidDown => resources::SQUID_DOWN,
            Item::Ice => resources::ICE,
            Item::MovableBlock => resources::ROCK,
            Item::Glove => resources::POW_GLOVE,
            Item::Teleporter => resources::TELEPORTER,
        }
    }

    pub fn map_entity_offset(&self) -> Vector2D<i32> {
        const STANDARD: Vector2D<i32> = Vector2D::new(0, -3);
        const ZERO: Vector2D<i32> = Vector2D::new(0, 0);

        match self {
            Item::Sword => STANDARD,
            Item::Slime => STANDARD,
            Item::Hero => STANDARD,
            Item::Stairs => ZERO,
            Item::Door => ZERO,
            Item::Key => STANDARD,
            Item::SwitchedOpenDoor => ZERO,
            Item::SwitchedClosedDoor => ZERO,
            Item::Switch => ZERO,
            Item::SwitchPressed => ZERO,
            Item::SpikesUp => ZERO,
            Item::SpikesDown => ZERO,
            Item::SquidUp => STANDARD,
            Item::SquidDown => STANDARD,
            Item::Ice => ZERO,
            Item::MovableBlock => ZERO,
            Item::Glove => STANDARD,
            Item::Teleporter => ZERO,
        }
    }
}

pub struct Entity(pub Item, pub Vector2D<i32>);

pub struct Level {
    pub map: Map<'static>,
    pub entities: &'static [Entity],
    pub directions: &'static [Direction],
    pub items: &'static [Item],
    pub name: &'static str,
}

impl Level {
    const fn new(
        map: Map<'static>,
        entities: &'static [Entity],
        directions: &'static [Direction],
        items: &'static [Item],
        name: &'static str,
    ) -> Self {
        Self {
            map,
            entities,
            directions,
            items,
            name,
        }
    }

    pub const fn get_level(level_number: usize) -> &'static Level {
        &levels::LEVELS[level_number]
    }

    pub const fn num_levels() -> usize {
        levels::LEVELS.len()
    }
}

mod levels {
    use super::*;
    use agb::fixnum::Vector2D;

    include!(concat!(env!("OUT_DIR"), "/levels.rs"));
}
