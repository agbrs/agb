use agb::{display::font::Font, include_aseprite, include_font};

include_aseprite!(mod sprites,
    "gfx/sprites16x16.aseprite",
    "gfx/sprites8x8.aseprite",
    "gfx/countdown.aseprite"
);

pub use sprites::*;

pub static FONT: Font = include_font!("fnt/yoster.ttf", 12);
