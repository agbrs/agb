//! This is a runnable example to show the various affine transformations on the Game Boy Advance.
#![no_main]
#![no_std]

extern crate alloc;

use alloc::{format, vec, vec::Vec};

use agb::{
    display::{
        GraphicsFrame, HEIGHT, Palette16, Rgb15, WIDTH,
        affine::{AffineMatrix, AffineMatrixObject},
        font::{AlignmentKind, Font, Layout, SpriteTextRenderer},
        object::{AffineMatrixInstance, AffineMode, Object, ObjectAffine, Size, SpriteVram},
        tiled::VRAM_MANAGER,
    },
    fixnum::{Num, Vector2D, num, vec2},
    include_aseprite, include_font,
    input::{Button, ButtonController},
};

include_aseprite!(mod sprites,
    "examples/gfx/crab.aseprite",
    "examples/gfx/box.aseprite",
);

static FONT: Font = include_font!("examples/font/ark-pixel-10px-proportional-ja.ttf", 10);

#[agb::entry]
fn entry(gba: agb::Gba) -> ! {
    main(gba);
}

fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.graphics.get();

    VRAM_MANAGER.set_background_palette_colour(0, 0, Rgb15::WHITE);

    let mut demonstration = AffineDemonstration::new();
    let mut button_controller = ButtonController::new();

    loop {
        button_controller.update();
        demonstration.update(&button_controller);

        let mut frame = gfx.frame();
        demonstration.show(&mut frame);
        frame.commit();
    }
}

struct AffineDemonstration {
    crab_sprite: SpriteVram,
    box_sprite: SpriteVram,

    description_objs: Vec<Object>,
    position_objs: Vec<Object>,
    text: Option<Layout>,

    text_renderer: SpriteTextRenderer,

    position: Vector2D<Num<i32, 8>>,
    demonstration: AffineTransformKind,
}

impl AffineDemonstration {
    fn new() -> Self {
        let crab_sprite = sprites::IDLE.sprite(0);
        let box_sprite = sprites::BOX.sprite(0);

        static PALETTE: Palette16 = const {
            let mut palette = [Rgb15::BLACK; 16];
            palette[1] = Rgb15::BLACK;
            Palette16::new(palette)
        };

        let text_renderer = SpriteTextRenderer::new((&PALETTE).into(), Size::S32x16);

        let demonstration = AffineTransformKind::default();

        Self {
            crab_sprite: crab_sprite.into(),
            box_sprite: box_sprite.into(),

            text: None,
            description_objs: vec![],
            position_objs: vec![],

            text_renderer,

            position: demonstration.start_position(),
            demonstration,
        }
    }

    fn update(&mut self, btn: &ButtonController) {
        if btn.is_just_pressed(Button::A) {
            self.demonstration = self.demonstration.next();
            self.text = None;
            self.position = self.demonstration.start_position();
        }

        self.position += btn.vector() * self.demonstration.values_step();

        if let Some(layout) = self.text.as_mut() {
            if let Some(lg) = layout.next() {
                self.description_objs
                    .push(self.text_renderer.show(&lg, (4, 4)));
            }
        } else {
            self.text = Some(Layout::new(
                &format!(
                    "{}\n\n\n\n\nA to change transform\nD-Pad to change x, y",
                    self.demonstration.text()
                ),
                &FONT,
                AlignmentKind::Left,
                32,
                1000,
            ));
            self.description_objs.clear();
        }

        let position_text = format!("x={}, y={}", self.position.x, self.position.y);
        let position_layout = Layout::new(&position_text, &FONT, AlignmentKind::Right, 32, 1000);
        self.position_objs = position_layout
            .map(|lg| self.text_renderer.show(&lg, (WIDTH - 8, 4)))
            .collect();
    }

    fn show(&self, frame: &mut GraphicsFrame) {
        let default_crab_pos = vec2((WIDTH - 32) / 3, (HEIGHT - 32) / 2);
        Object::new(self.crab_sprite.clone())
            .set_pos(default_crab_pos)
            .show(frame);
        Object::new(self.box_sprite.clone())
            .set_pos(default_crab_pos)
            .show(frame);

        let matrix = self.demonstration.matrix(self.position);

        let obj_matrix =
            AffineMatrixInstance::new(AffineMatrixObject::from_affine_wrapping(matrix));

        ObjectAffine::new(
            self.crab_sprite.clone(),
            obj_matrix,
            AffineMode::AffineDouble,
        )
        .set_pos(
            vec2(2 * (WIDTH - 32) / 3, (HEIGHT - 32) / 2) + matrix.position().round()
                - vec2(16, 16),
        )
        .show(frame);
        Object::new(self.box_sprite.clone())
            .set_pos((2 * (WIDTH - 32) / 3, (HEIGHT - 32) / 2))
            .show(frame);

        for description_obj in &self.description_objs {
            description_obj.show(frame);
        }
        for position_obj in &self.position_objs {
            position_obj.show(frame);
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum AffineTransformKind {
    #[default]
    Translation,
    Scale,
    Rotation,
    Shear,
}

impl AffineTransformKind {
    fn text(self) -> &'static str {
        match self {
            AffineTransformKind::Translation => "Translation\nMoving the object around",
            AffineTransformKind::Scale => "Scaling\nSmaller numbers result in larger objects",
            AffineTransformKind::Rotation => {
                "Rotation\nOnly x works here, positive is anti-clockwise"
            }
            AffineTransformKind::Shear => "Shearing\nLarger numbers shear more",
        }
    }

    fn values_step(self) -> Num<i32, 8> {
        match self {
            AffineTransformKind::Translation => num!(1.5),
            AffineTransformKind::Scale => Num::from_raw(5),
            AffineTransformKind::Rotation => Num::from_raw(3),
            AffineTransformKind::Shear => Num::from_raw(3),
        }
    }

    fn start_position(self) -> Vector2D<Num<i32, 8>> {
        match self {
            AffineTransformKind::Translation => vec2(num!(0), num!(0)),
            AffineTransformKind::Scale => vec2(num!(1), num!(1)),
            AffineTransformKind::Rotation => vec2(num!(0), num!(0)),
            AffineTransformKind::Shear => vec2(num!(0), num!(0)),
        }
    }

    fn matrix(self, values: Vector2D<Num<i32, 8>>) -> AffineMatrix<Num<i32, 8>> {
        match self {
            AffineTransformKind::Translation => {
                // spin in a circle
                AffineMatrix::from_translation(values)
            }
            AffineTransformKind::Scale => {
                // scale up and down but opposite on the x and y axes
                AffineMatrix::from_scale(values)
            }
            AffineTransformKind::Rotation => AffineMatrix::from_rotation(values.x),
            AffineTransformKind::Shear => AffineMatrix::from_shear(values),
        }
    }

    fn next(self) -> Self {
        match self {
            AffineTransformKind::Translation => AffineTransformKind::Scale,
            AffineTransformKind::Scale => AffineTransformKind::Rotation,
            AffineTransformKind::Rotation => AffineTransformKind::Shear,
            AffineTransformKind::Shear => AffineTransformKind::Translation,
        }
    }
}
