use agb::{
    display::{object::Graphics, Font},
    include_aseprite, include_font,
};

static SPRITES: &Graphics = include_aseprite!(
    "gfx/sprites16x16.aseprite",
    "gfx/sprites8x8.aseprite",
    "gfx/countdown.aseprite"
);

macro_rules! named_tag {
    (
        $sprites:ident, [
            $($name:tt),+ $(,)?
        ] $(,)?
    ) => {
        $(
            pub static $name: &agb::display::object::Tag = $sprites.tags().get(stringify!($name));
        )+
    };
}

named_tag!(
    SPRITES,
    [
        SWORD,
        SWORD_SHADOW,
        SLIME,
        SLIME_SHADOW,
        STAIRS,
        HERO,
        HERO_CARRY,
        ARROW_LEFT,
        ARROW_RIGHT,
        ARROW_UP,
        ARROW_DOWN,
        CURSOR,
        KEY,
        KEY_SHADOW,
        DOOR,
        SWITCHED_DOOR_CLOSED,
        SWITCHED_DOOR_OPEN,
        SPIKES_ON,
        SPIKES_OFF,
        BUTTON_ON,
        BUTTON_OFF,
        SQUID_UP,
        SQUID_DOWN,
        SQUID_UP_SHADOW,
        SQUID_DOWN_SHADOW,
    ]
);

pub static FONT: Font = include_font!("fnt/yoster.ttf", 12);
