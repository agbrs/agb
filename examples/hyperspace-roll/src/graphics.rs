use agb::{
    display::object::{OamManaged, Object, Sprite, Tag},
    fixnum::Vector2D,
};
use alloc::vec::Vec;

use crate::{EnemyAttackType, Face, Ship};

const SPRITES: &agb::display::object::Graphics = agb::include_aseprite!(
    "gfx/dice-faces.aseprite",
    "gfx/ships.aseprite",
    "gfx/small-sprites.aseprite"
);
pub const FACE_SPRITES: &FaceSprites = &FaceSprites::load_face_sprites();
pub const ENEMY_ATTACK_SPRITES: &EnemyAttackSprites = &EnemyAttackSprites::new();
pub const SELECT_BOX: &Tag = SPRITES.tags().get("selection");
pub const SELECTED_BOX: &Sprite = SPRITES.tags().get("selected").sprite(0);
pub const MODIFIED_BOX: &Sprite = SPRITES.tags().get("modified").sprite(0);

pub const BULLET_SPRITE: &Sprite = SPRITES.tags().get("bullet").sprite(0);
pub const DISRUPT_BULLET: &Sprite = SPRITES.tags().get("disrupt bullet").sprite(0);
pub const BURST_BULLET: &Sprite = SPRITES.tags().get("burst shield bullet").sprite(0);
pub const SHIELD: &Tag = SPRITES.tags().get("ship shield");

pub const SHIP_SPRITES: &ShipSprites = &ShipSprites::load_ship_sprites();

pub const SMALL_SPRITES: &SmallSprites = &SmallSprites {};

pub struct FaceSprites {
    sprites: [&'static Sprite; 17],
}

impl FaceSprites {
    const fn load_face_sprites() -> Self {
        const S_SHOOT: &Sprite = SPRITES.tags().get("shoot").sprite(0);
        const S_SHIELD: &Sprite = SPRITES.tags().get("shield").sprite(0);
        const S_MALFUNCTION: &Sprite = SPRITES.tags().get("malfunction").sprite(0);
        const S_HEAL: &Sprite = SPRITES.tags().get("player_heal").sprite(0);
        const S_BYPASS: &Sprite = SPRITES.tags().get("shield bypass").sprite(0);
        const S_DOUBLE_SHOT: &Sprite = SPRITES.tags().get("double shoot").sprite(0);
        const S_TRIPLE_SHOT: &Sprite = SPRITES.tags().get("triple shoot").sprite(0);
        const S_BLANK: &Sprite = SPRITES.tags().get("blank").sprite(0);
        const S_DISRUPT: &Sprite = SPRITES.tags().get("disruption").sprite(0);
        const S_MALFUNCTION_SHOOT: &Sprite = SPRITES.tags().get("malfunction shot").sprite(0);
        const S_DOUBLE_SHIELD: &Sprite = SPRITES.tags().get("double shield").sprite(0);
        const S_TRIPLE_SHIELD: &Sprite = SPRITES.tags().get("triple shield").sprite(0);
        const S_DOUBLE_SHIELD_VALUE: &Sprite = SPRITES.tags().get("double shield value").sprite(0);
        const S_DOUBLE_SHOT_VALUE: &Sprite = SPRITES.tags().get("double shoot power").sprite(0);
        const S_TRIPLE_SHOT_VALUE: &Sprite = SPRITES.tags().get("triple shoot power").sprite(0);
        const S_BURST_SHIELD: &Sprite = SPRITES.tags().get("burst shield").sprite(0);
        const S_INVERT: &Sprite = SPRITES.tags().get("swap shield and shoot").sprite(0);

        Self {
            sprites: [
                S_SHOOT,
                S_SHIELD,
                S_MALFUNCTION,
                S_HEAL,
                S_BYPASS,
                S_DOUBLE_SHOT,
                S_TRIPLE_SHOT,
                S_BLANK,
                S_DISRUPT,
                S_MALFUNCTION_SHOOT,
                S_DOUBLE_SHIELD,
                S_TRIPLE_SHIELD,
                S_DOUBLE_SHIELD_VALUE,
                S_DOUBLE_SHOT_VALUE,
                S_TRIPLE_SHOT_VALUE,
                S_BURST_SHIELD,
                S_INVERT,
            ],
        }
    }

    pub fn sprite_for_face(&self, face: Face) -> &'static Sprite {
        self.sprites[face as usize]
    }
}

pub struct ShipSprites {
    sprites: [&'static Sprite; 4],
}

impl ShipSprites {
    const fn load_ship_sprites() -> Self {
        const S_PLAYER: &Sprite = SPRITES.tags().get("player").sprite(0);
        const S_DRONE: &Sprite = SPRITES.tags().get("drone").sprite(0);
        const S_PILOTED_SHIP: &Sprite = SPRITES.tags().get("piloted ship").sprite(0);
        const S_SHIELD: &Sprite = SPRITES.tags().get("ship shield").sprite(0);

        Self {
            sprites: [S_PLAYER, S_DRONE, S_PILOTED_SHIP, S_SHIELD],
        }
    }

    pub fn sprite_for_ship(&self, ship: Ship) -> &'static Sprite {
        self.sprites[ship as usize]
    }
}

pub struct SmallSprites;

impl SmallSprites {
    pub const fn number(&self, i: u32) -> &'static Sprite {
        SPRITES.tags().get("numbers").sprite(i as usize)
    }

    pub const fn slash(&self) -> &'static Sprite {
        SPRITES.tags().get("numbers").sprite(10)
    }

    pub const fn red_bar(&self, i: usize) -> &'static Sprite {
        SPRITES.tags().get("red bar").sprite(i)
    }
}

pub struct EnemyAttackSprites {
    sprites: [&'static Sprite; 3],
}

impl EnemyAttackSprites {
    const fn new() -> Self {
        const S_SHOOT: &Sprite = SPRITES.tags().get("enemy shoot").sprite(0);
        const S_SHIELD: &Sprite = SPRITES.tags().get("enemy shield").sprite(0);
        const S_HEAL: &Sprite = SPRITES.tags().get("enemy heal").sprite(0);

        Self {
            sprites: [S_SHOOT, S_SHIELD, S_HEAL],
        }
    }

    pub fn sprite_for_attack(&self, attack: EnemyAttackType) -> &'static Sprite {
        self.sprites[attack as usize]
    }
}

pub struct HealthBar<'a> {
    max: usize,
    sprites: Vec<Object<'a>>,
}

impl<'a> HealthBar<'a> {
    pub fn new(pos: Vector2D<i32>, max: usize, obj: &'a OamManaged) -> Self {
        assert_eq!(max % 8, 0);

        let sprites = (0..(max / 8))
            .map(|i| {
                let mut health_object = obj.object_sprite(SMALL_SPRITES.red_bar(0));
                health_object
                    .set_position(pos + (i as i32 * 8, 0).into())
                    .show();
                health_object
            })
            .collect();

        Self { max, sprites }
    }

    pub fn set_value(&mut self, new_value: usize, obj: &'a OamManaged) {
        assert!(new_value <= self.max);

        for (i, sprite) in self.sprites.iter_mut().enumerate() {
            if (i + 1) * 8 < new_value {
                sprite.set_sprite(obj.get_sprite(SMALL_SPRITES.red_bar(0)));
            } else if i * 8 < new_value {
                sprite.set_sprite(obj.get_sprite(SMALL_SPRITES.red_bar(8 - (new_value - i * 8))));
            } else {
                sprite.set_sprite(obj.get_sprite(SMALL_SPRITES.red_bar(8)));
            }
        }
    }

    pub fn show(&mut self) {
        for obj in self.sprites.iter_mut() {
            obj.show();
        }
    }

    pub fn hide(&mut self) {
        for obj in self.sprites.iter_mut() {
            obj.hide();
        }
    }
}

pub struct FractionDisplay<'a> {
    sprites: Vec<Object<'a>>,
    digits: usize,

    current_current: usize,
    current_max: usize,
}

impl<'a> FractionDisplay<'a> {
    pub fn new(pos: Vector2D<i32>, digits: usize, obj: &'a OamManaged) -> Self {
        let mut sprites = Vec::with_capacity(digits * 2 + 1);

        for i in 0..digits {
            let mut left_digit = obj.object_sprite(SMALL_SPRITES.number(0));
            left_digit.set_position(pos + (i as i32 * 4, 0).into());

            sprites.push(left_digit);

            let mut right_digit = obj.object_sprite(SMALL_SPRITES.number(0));
            right_digit.set_position(pos + (i as i32 * 4 + digits as i32 * 4 + 7, 0).into());

            sprites.push(right_digit);
        }

        let mut slash = obj.object_sprite(SMALL_SPRITES.slash());
        slash.set_position(pos + (digits as i32 * 4 + 1, 0).into());
        sprites.push(slash);

        Self {
            sprites,
            digits,
            current_current: 0,
            current_max: 0,
        }
    }

    pub fn set_value(&mut self, current: usize, max: usize, obj: &'a OamManaged) {
        if self.current_current == current && self.current_max == max {
            return;
        }

        let mut current = current;
        let mut max = max;

        for i in 0..self.digits {
            let current_value_digit = current % 10;
            current /= 10;
            let current_value_sprite = &mut self.sprites[(self.digits - i) * 2 - 2];
            current_value_sprite
                .set_sprite(obj.get_sprite(SMALL_SPRITES.number(current_value_digit as u32)));

            let max_value_digit = max % 10;
            max /= 10;
            let max_value_sprite = &mut self.sprites[(self.digits - i) * 2 - 1];
            max_value_sprite
                .set_sprite(obj.get_sprite(SMALL_SPRITES.number(max_value_digit as u32)));
        }
    }
}

pub struct NumberDisplay<'a> {
    objects: Vec<Object<'a>>,
    value: Option<u32>,
    position: Vector2D<i32>,
}

impl<'a> NumberDisplay<'a> {
    pub fn new(position: Vector2D<i32>) -> Self {
        Self {
            objects: Vec::new(),
            value: None,
            position,
        }
    }

    pub fn set_value(&mut self, new_value: Option<u32>, obj: &'a OamManaged) {
        if self.value == new_value {
            return;
        }

        self.value = new_value;

        self.objects.clear();

        if let Some(mut new_value) = new_value {
            if new_value == 0 {
                let mut zero_object = obj.object_sprite(SMALL_SPRITES.number(0));
                zero_object.show().set_position(self.position);

                self.objects.push(zero_object);
                return;
            }

            let mut digit = 0;
            while new_value != 0 {
                let current_value_digit = new_value % 10;
                new_value /= 10;

                let mut current_value_obj =
                    obj.object_sprite(SMALL_SPRITES.number(current_value_digit));

                current_value_obj
                    .show()
                    .set_position(self.position - (digit * 4, 0).into());

                digit += 1;

                self.objects.push(current_value_obj);
            }
        }
    }
}
