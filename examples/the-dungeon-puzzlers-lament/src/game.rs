use agb::{
    display::{
        object::{
            OamIterator, ObjectTextRender, ObjectUnmanaged, PaletteVram, Size, SpriteLoader,
            SpriteVram, TextAlignment,
        },
        palette16::Palette16,
        tiled::{BackgroundIterator, RegularBackgroundSize, RegularBackgroundTiles, TileFormat},
        Priority, HEIGHT,
    },
    fixnum::Vector2D,
    input::{Button, ButtonController, Tri},
};

use crate::{
    resources::{ARROW_RIGHT, FONT},
    sfx::Sfx,
};

use self::{
    game_state::{GameState, PLAY_AREA_HEIGHT, PLAY_AREA_WIDTH},
    simulation::Simulation,
};

pub use simulation::Direction;

use core::{cell::RefCell, fmt::Write};

mod game_state;
mod simulation;

mod numbers;

struct Game {
    phase: GamePhase,
}

struct Lament {
    level: usize,
    writer: RefCell<ObjectTextRender<'static>>,
    background: RegularBackgroundTiles,
}

fn generate_text_palette() -> PaletteVram {
    let mut palette = [0x0; 16];
    palette[1] = 0xFF_FF;
    let palette = Palette16::new(palette);
    PaletteVram::new(&palette).unwrap()
}

impl Lament {
    fn new(level: usize) -> Self {
        let palette = generate_text_palette();

        let mut writer = ObjectTextRender::new(&super::resources::FONT, Size::S16x16, palette);

        let _ = writeln!(
            writer,
            "{}\n\n{}",
            numbers::NUMBERS[level],
            crate::level::Level::get_level(level).name
        );

        writer.layout(
            Vector2D::new(
                PLAY_AREA_WIDTH as i32 * 16 - 32,
                PLAY_AREA_HEIGHT as i32 * 16,
            ),
            TextAlignment::Center,
            0,
        );

        Self {
            level,
            writer: RefCell::new(writer),
            background: RegularBackgroundTiles::new(
                Priority::P1,
                RegularBackgroundSize::Background32x32,
                TileFormat::FourBpp,
            ),
        }
    }

    fn update(mut self, input: &ButtonController) -> GamePhase {
        {
            let mut writer = self.writer.borrow_mut();
            writer.next_letter_group();
            writer.update(Vector2D::new(16, HEIGHT / 4));

            self.background.commit();
        }
        if input.is_just_pressed(Button::A) {
            GamePhase::Construction(Construction::new(self.level))
        } else {
            GamePhase::Lament(self)
        }
    }

    fn render(&self, oam: &mut OamIterator, bg_iter: &mut BackgroundIterator) {
        self.writer.borrow_mut().commit(oam);
        self.background.show(bg_iter);
    }
}

struct Construction {
    game: GameState,
    background: RegularBackgroundTiles,
}

impl Construction {
    fn new(level: usize) -> Self {
        let game = GameState::new(level);
        let mut background = RegularBackgroundTiles::new(
            Priority::P1,
            RegularBackgroundSize::Background32x32,
            TileFormat::FourBpp,
        );

        game.load_level_background(&mut background);
        background.commit();

        Self { background, game }
    }

    fn update(
        mut self,
        input: &ButtonController,
        sfx: &mut Sfx,
        loader: &mut SpriteLoader,
    ) -> GamePhase {
        self.game.step(input, sfx);
        if input.is_just_pressed(Button::START) {
            self.game.force_place();
            GamePhase::Execute(Execute::new(self, sfx, loader))
        } else {
            GamePhase::Construction(self)
        }
    }

    fn render(
        &self,
        oam: &mut OamIterator,
        loader: &mut SpriteLoader,
        bg_iter: &mut BackgroundIterator,
    ) {
        self.game.render(loader, oam);
        self.background.show(bg_iter);
    }
}

impl Execute {
    fn new(construction: Construction, sfx: &mut Sfx, loader: &mut SpriteLoader) -> Self {
        Self {
            simulation: construction.game.create_simulation(sfx, loader),
            construction,
        }
    }

    fn update(
        mut self,
        input: &ButtonController,
        sfx: &mut Sfx,
        loader: &mut SpriteLoader,
    ) -> GamePhase {
        if input.is_just_pressed(Button::START) {
            return GamePhase::Construction(self.construction);
        }

        match self.simulation.update(loader, sfx) {
            simulation::Outcome::Continue => GamePhase::Execute(self),
            simulation::Outcome::Loss => GamePhase::Construction(self.construction),
            simulation::Outcome::Win => GamePhase::NextLevel,
        }
    }

    fn render(
        &self,
        loader: &mut SpriteLoader,
        oam: &mut OamIterator,
        bg_iter: &mut BackgroundIterator,
    ) {
        self.simulation.render(oam);
        self.construction
            .game
            .render_arrows(loader, oam, Some(self.simulation.current_turn()));
        self.construction.background.show(bg_iter);
    }
}

struct Execute {
    simulation: Simulation,
    construction: Construction,
}

#[derive(Default)]
enum GamePhase {
    #[default]
    Empty,
    Lament(Lament),
    Construction(Construction),
    Execute(Execute),
    NextLevel,
}

impl GamePhase {
    fn update(&mut self, input: &ButtonController, sfx: &mut Sfx, loader: &mut SpriteLoader) {
        *self = match core::mem::take(self) {
            GamePhase::Lament(lament) => lament.update(input),
            GamePhase::Construction(construction) => construction.update(input, sfx, loader),
            GamePhase::Execute(execute) => execute.update(input, sfx, loader),
            GamePhase::NextLevel => GamePhase::NextLevel,
            GamePhase::Empty => panic!("bad state"),
        }
    }

    fn render(
        &self,
        loader: &mut SpriteLoader,
        oam: &mut OamIterator,
        bg_iter: &mut BackgroundIterator,
    ) {
        match self {
            GamePhase::Empty => panic!("bad state"),
            GamePhase::Lament(lament) => lament.render(oam, bg_iter),
            GamePhase::Construction(construction) => construction.render(oam, loader, bg_iter),
            GamePhase::Execute(execute) => execute.render(loader, oam, bg_iter),
            GamePhase::NextLevel => {}
        }
    }
}

impl Game {
    pub fn new(level: usize) -> Self {
        Self {
            phase: GamePhase::Lament(Lament::new(level)),
        }
    }

    pub fn update(
        &mut self,
        input: &ButtonController,
        sfx: &mut Sfx,
        loader: &mut SpriteLoader,
    ) -> bool {
        self.phase.update(input, sfx, loader);
        matches!(self.phase, GamePhase::NextLevel)
    }

    pub fn render(
        &self,
        loader: &mut SpriteLoader,
        oam: &mut OamIterator,
        bg_iter: &mut BackgroundIterator,
    ) {
        self.phase.render(loader, oam, bg_iter)
    }
}

pub struct Pausable {
    paused: Paused,
    menu: PauseMenu,
    game: Game,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Paused {
    Paused,
    Playing,
}

impl Paused {
    fn change(self) -> Paused {
        match self {
            Paused::Paused => Paused::Playing,
            Paused::Playing => Paused::Paused,
        }
    }
}

#[derive(Clone, Copy)]
pub enum PauseSelection {
    Restart,
    LevelSelect(usize),
}

enum PauseSelectionInner {
    Restart,
    LevelSelect,
}

struct PauseMenu {
    option_text: RefCell<[ObjectTextRender<'static>; 2]>,
    selection: PauseSelectionInner,
    indicator_sprite: SpriteVram,
    selected_level: usize,
    maximum_level: usize,
}

impl PauseMenu {
    fn text_at_position(
        text: core::fmt::Arguments,
        position: Vector2D<i32>,
    ) -> ObjectTextRender<'static> {
        let mut t = ObjectTextRender::new(&FONT, Size::S32x16, generate_text_palette());

        let _ = writeln!(t, "{}", text);
        t.layout(Vector2D::new(i32::MAX, i32::MAX), TextAlignment::Left, 0);
        t.next_line();
        t.update(position);
        t
    }

    fn new(loader: &mut SpriteLoader, maximum_level: usize, current_level: usize) -> Self {
        PauseMenu {
            option_text: RefCell::new([
                Self::text_at_position(format_args!("Restart"), Vector2D::new(32, HEIGHT / 4)),
                Self::text_at_position(
                    format_args!("Go to level: {}", current_level + 1),
                    Vector2D::new(32, HEIGHT / 4 + 20),
                ),
            ]),
            selection: PauseSelectionInner::Restart,
            indicator_sprite: loader.get_vram_sprite(ARROW_RIGHT.sprite(0)),
            selected_level: current_level,
            maximum_level,
        }
    }

    fn update(&mut self, input: &ButtonController) -> Option<PauseSelection> {
        if input.is_just_pressed(Button::UP) | input.is_just_pressed(Button::DOWN) {
            self.selection = match self.selection {
                PauseSelectionInner::Restart => PauseSelectionInner::LevelSelect,
                PauseSelectionInner::LevelSelect => PauseSelectionInner::Restart,
            };
        }

        let lr = Tri::from((
            input.is_just_pressed(Button::LEFT),
            input.is_just_pressed(Button::RIGHT),
        ));
        if matches!(self.selection, PauseSelectionInner::LevelSelect) && lr != Tri::Zero {
            let selected_level = self.selected_level as i32;
            let selected_level =
                (selected_level + lr as i32).rem_euclid(self.maximum_level as i32 + 1);
            self.selected_level = selected_level as usize;
            self.option_text.borrow_mut()[1] = Self::text_at_position(
                format_args!("Go to level: {}", selected_level + 1),
                Vector2D::new(32, HEIGHT / 4 + 20),
            )
        }

        if input.is_just_pressed(Button::A) | input.is_just_pressed(Button::START) {
            Some(match self.selection {
                PauseSelectionInner::Restart => PauseSelection::Restart,
                PauseSelectionInner::LevelSelect => {
                    PauseSelection::LevelSelect(self.selected_level)
                }
            })
        } else {
            None
        }
    }

    fn render(&self, oam: &mut OamIterator, _bg_iter: &mut BackgroundIterator) {
        for text in self.option_text.borrow_mut().iter_mut() {
            text.commit(oam);
        }
        let mut indicator = ObjectUnmanaged::new(self.indicator_sprite.clone());
        indicator.show();
        match self.selection {
            PauseSelectionInner::Restart => indicator.set_position(Vector2D::new(16, HEIGHT / 4)),
            PauseSelectionInner::LevelSelect => {
                indicator.set_position(Vector2D::new(16, HEIGHT / 4 + 20))
            }
        };
        if let Some(slot) = oam.next() {
            slot.set(&indicator);
        }
    }
}

pub enum UpdateResult {
    MenuSelection(PauseSelection),
    NextLevel,
}

impl Pausable {
    pub fn new(level: usize, maximum_level: usize, loader: &mut SpriteLoader) -> Self {
        Self {
            paused: Paused::Playing,
            game: Game::new(level),
            menu: PauseMenu::new(loader, maximum_level, level),
        }
    }

    pub fn update(
        &mut self,
        input: &ButtonController,
        sfx: &mut Sfx,
        loader: &mut SpriteLoader,
    ) -> Option<UpdateResult> {
        if input.is_just_pressed(Button::SELECT)
            || (matches!(self.paused, Paused::Paused) && input.is_just_pressed(Button::B))
        {
            self.paused = self.paused.change();
        }

        if !matches!(self.paused, Paused::Paused) {
            if self.game.update(input, sfx, loader) {
                Some(UpdateResult::NextLevel)
            } else {
                None
            }
        } else {
            self.menu.update(input).map(UpdateResult::MenuSelection)
        }
    }

    pub fn render(
        &self,
        loader: &mut SpriteLoader,
        oam: &mut OamIterator,
        bg_iter: &mut BackgroundIterator,
    ) {
        if matches!(self.paused, Paused::Paused) {
            self.menu.render(oam, bg_iter);
        } else {
            self.game.render(loader, oam, bg_iter);
        }
    }
}
