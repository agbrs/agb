use agb::{
    display::{
        GraphicsFrame,
        object::{Object, Sprite, Tag},
    },
    fixnum::Vector2D,
};
use alloc::vec::Vec;

use crate::{EnemyAttackType, Face, Ship};

agb::include_aseprite!(mod sprites,
    "gfx/dice-faces.aseprite",
    "gfx/ships.aseprite",
    "gfx/small-sprites.aseprite"
);
pub static FACE_SPRITES: &FaceSprites = {
    static S_SHOOT: &Sprite = sprites::SHOOT.sprite(0);
    static S_SHIELD: &Sprite = sprites::SHIELD.sprite(0);
    static S_MALFUNCTION: &Sprite = sprites::MALFUNCTION.sprite(0);
    static S_HEAL: &Sprite = sprites::PLAYER_HEAL.sprite(0);
    static S_BYPASS: &Sprite = sprites::SHIELD_BYPASS.sprite(0);
    static S_DOUBLE_SHOT: &Sprite = sprites::DOUBLE_SHOOT.sprite(0);
    static S_TRIPLE_SHOT: &Sprite = sprites::TRIPLE_SHOOT.sprite(0);
    static S_BLANK: &Sprite = sprites::BLANK.sprite(0);
    static S_DISRUPT: &Sprite = sprites::DISRUPTION.sprite(0);
    static S_MALFUNCTION_SHOOT: &Sprite = sprites::MALFUNCTION_SHOT.sprite(0);
    static S_DOUBLE_SHIELD: &Sprite = sprites::DOUBLE_SHIELD.sprite(0);
    static S_TRIPLE_SHIELD: &Sprite = sprites::TRIPLE_SHIELD.sprite(0);
    static S_DOUBLE_SHIELD_VALUE: &Sprite = sprites::DOUBLE_SHIELD_VALUE.sprite(0);
    static S_DOUBLE_SHOT_VALUE: &Sprite = sprites::DOUBLE_SHOOT_POWER.sprite(0);
    static S_TRIPLE_SHOT_VALUE: &Sprite = sprites::TRIPLE_SHOOT_POWER.sprite(0);
    static S_BURST_SHIELD: &Sprite = sprites::BURST_SHIELD.sprite(0);
    static S_INVERT: &Sprite = sprites::SWAP_SHIELD_AND_SHOOT.sprite(0);

    &FaceSprites {
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
};
pub static ENEMY_ATTACK_SPRITES: &EnemyAttackSprites = {
    static S_SHOOT: &Sprite = sprites::ENEMY_SHOOT.sprite(0);
    static S_SHIELD: &Sprite = sprites::ENEMY_SHIELD.sprite(0);
    static S_HEAL: &Sprite = sprites::ENEMY_HEAL.sprite(0);

    &EnemyAttackSprites {
        sprites: [S_SHOOT, S_SHIELD, S_HEAL],
    }
};
pub static SELECT_BOX: &Tag = &sprites::SELECTION;
pub static SELECTED_BOX: &Sprite = sprites::SELECTED.sprite(0);
pub static MODIFIED_BOX: &Sprite = sprites::MODIFIED.sprite(0);

pub static BULLET_SPRITE: &Sprite = sprites::BULLET.sprite(0);
pub static DISRUPT_BULLET: &Sprite = sprites::DISRUPT_BULLET.sprite(0);
pub static BURST_BULLET: &Sprite = sprites::BURST_SHIELD_BULLET.sprite(0);
pub static SHIELD: &Tag = &sprites::SHIP_SHIELD;

pub static SHIP_SPRITES: &ShipSprites = {
    static S_PLAYER: &Sprite = sprites::PLAYER.sprite(0);
    static S_DRONE: &Sprite = sprites::DRONE.sprite(0);
    static S_PILOTED_SHIP: &Sprite = sprites::PILOTED_SHIP.sprite(0);
    static S_SHIELD: &Sprite = SHIELD.sprite(0);

    &ShipSprites {
        sprites: [S_PLAYER, S_DRONE, S_PILOTED_SHIP, S_SHIELD],
    }
};

pub static SMALL_SPRITES: &SmallSprites = &SmallSprites {};

pub struct FaceSprites {
    sprites: [&'static Sprite; 17],
}

impl FaceSprites {
    pub fn sprite_for_face(&self, face: Face) -> &'static Sprite {
        self.sprites[face as usize]
    }
}

pub struct ShipSprites {
    sprites: [&'static Sprite; 4],
}

impl ShipSprites {
    pub fn sprite_for_ship(&self, ship: Ship) -> &'static Sprite {
        self.sprites[ship as usize]
    }
}

pub struct SmallSprites;

impl SmallSprites {
    pub fn number(&self, i: u32) -> &'static Sprite {
        sprites::NUMBERS.sprite(i as usize)
    }

    pub fn slash(&self) -> &'static Sprite {
        sprites::NUMBERS.sprite(10)
    }

    pub fn red_bar(&self, i: usize) -> &'static Sprite {
        sprites::RED_BAR.sprite(i)
    }
}

pub struct EnemyAttackSprites {
    sprites: [&'static Sprite; 3],
}

impl EnemyAttackSprites {
    pub fn sprite_for_attack(&self, attack: EnemyAttackType) -> &'static Sprite {
        self.sprites[attack as usize]
    }
}

pub struct HealthBar {
    max: usize,
    sprites: Vec<Object>,
}

impl HealthBar {
    pub fn new(pos: Vector2D<i32>, max: usize) -> Self {
        assert_eq!(max % 8, 0);

        let sprites = (0..(max / 8))
            .map(|i| {
                let mut health_object = Object::new(SMALL_SPRITES.red_bar(0));
                health_object.set_position(pos + (i as i32 * 8, 0).into());
                health_object
            })
            .collect();

        Self { max, sprites }
    }

    pub fn set_value(&mut self, new_value: usize) {
        assert!(new_value <= self.max);

        for (i, sprite) in self.sprites.iter_mut().enumerate() {
            if (i + 1) * 8 < new_value {
                sprite.set_sprite(SMALL_SPRITES.red_bar(0));
            } else if i * 8 < new_value {
                sprite.set_sprite(SMALL_SPRITES.red_bar(8 - (new_value - i * 8)));
            } else {
                sprite.set_sprite(SMALL_SPRITES.red_bar(8));
            }
        }
    }

    pub fn show(&mut self, frame: &mut GraphicsFrame) {
        for obj in self.sprites.iter_mut() {
            obj.show(frame);
        }
    }
}

pub struct FractionDisplay {
    sprites: Vec<Object>,
    digits: usize,

    current_current: usize,
    current_max: usize,
}

impl FractionDisplay {
    pub fn new(pos: Vector2D<i32>, digits: usize) -> Self {
        let mut sprites = Vec::with_capacity(digits * 2 + 1);

        for i in 0..digits {
            let mut left_digit = Object::new(SMALL_SPRITES.number(0));
            left_digit.set_position(pos + (i as i32 * 4, 0).into());

            sprites.push(left_digit);

            let mut right_digit = Object::new(SMALL_SPRITES.number(0));
            right_digit.set_position(pos + (i as i32 * 4 + digits as i32 * 4 + 7, 0).into());

            sprites.push(right_digit);
        }

        let mut slash = Object::new(SMALL_SPRITES.slash());
        slash.set_position(pos + (digits as i32 * 4 + 1, 0).into());
        sprites.push(slash);

        Self {
            sprites,
            digits,
            current_current: 0,
            current_max: 0,
        }
    }

    pub fn set_value(&mut self, current: usize, max: usize) {
        if self.current_current == current && self.current_max == max {
            return;
        }

        let mut current = current;
        let mut max = max;

        for i in 0..self.digits {
            let current_value_digit = current % 10;
            current /= 10;
            let current_value_sprite = &mut self.sprites[(self.digits - i) * 2 - 2];
            current_value_sprite.set_sprite(SMALL_SPRITES.number(current_value_digit as u32));

            let max_value_digit = max % 10;
            max /= 10;
            let max_value_sprite = &mut self.sprites[(self.digits - i) * 2 - 1];
            max_value_sprite.set_sprite(SMALL_SPRITES.number(max_value_digit as u32));
        }
    }

    pub fn show(&self, oam_frame: &mut GraphicsFrame) {
        for sprite in self.sprites.iter() {
            sprite.show(oam_frame);
        }
    }
}

pub struct NumberDisplay {
    objects: Vec<Object>,
    value: Option<u32>,
    position: Vector2D<i32>,
}

impl NumberDisplay {
    pub fn new(position: Vector2D<i32>) -> Self {
        Self {
            objects: Vec::new(),
            value: None,
            position,
        }
    }

    pub fn set_value(&mut self, new_value: Option<u32>) {
        if self.value == new_value {
            return;
        }

        self.value = new_value;

        self.objects.clear();

        if let Some(mut new_value) = new_value {
            if new_value == 0 {
                let mut zero_object = Object::new(SMALL_SPRITES.number(0));
                zero_object.set_position(self.position);

                self.objects.push(zero_object);
                return;
            }

            let mut digit = 0;
            while new_value != 0 {
                let current_value_digit = new_value % 10;
                new_value /= 10;

                let mut current_value_obj = Object::new(SMALL_SPRITES.number(current_value_digit));

                current_value_obj.set_position(self.position - (digit * 4, 0).into());

                digit += 1;

                self.objects.push(current_value_obj);
            }
        }
    }

    pub fn show(&self, frame: &mut GraphicsFrame) {
        for obj in self.objects.iter() {
            obj.show(frame);
        }
    }
}
