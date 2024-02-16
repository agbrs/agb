use agb::{
    display::{
        object::{
            OamIterator, ObjectTextRender, ObjectUnmanaged, PaletteVram, Size, SpriteLoader,
            SpriteVram, TextAlignment,
        },
        palette16::Palette16,
        tiled::{MapLoan, RegularMap, TiledMap, VRamManager},
        HEIGHT,
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

struct Game<'a, 'b> {
    phase: GamePhase<'a, 'b>,
}

struct Lament<'a, 'b> {
    level: usize,
    writer: RefCell<ObjectTextRender<'static>>,
    background: &'a mut MapLoan<'b, RegularMap>,
}

fn generate_text_palette() -> PaletteVram {
    let mut palette = [0x0; 16];
    palette[1] = 0xFF_FF;
    let palette = Palette16::new(palette);
    PaletteVram::new(&palette).unwrap()
}

impl<'a, 'b> Lament<'a, 'b> {
    fn new(level: usize, background: &'a mut MapLoan<'b, RegularMap>) -> Self {
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
            background,
        }
    }

    fn update(self, input: &ButtonController, vram_manager: &mut VRamManager) -> GamePhase<'a, 'b> {
        {
            let mut writer = self.writer.borrow_mut();
            writer.next_letter_group();
            writer.update(Vector2D::new(16, HEIGHT / 4));
        }
        if input.is_just_pressed(Button::A) {
            GamePhase::Construction(Construction::new(self.level, self.background, vram_manager))
        } else {
            GamePhase::Lament(self)
        }
    }

    fn render(&self, oam: &mut OamIterator) {
        self.writer.borrow_mut().commit(oam);
    }
}

struct Construction<'a, 'b> {
    game: GameState,
    background: &'a mut MapLoan<'b, RegularMap>,
}

impl<'a, 'b> Drop for Construction<'a, 'b> {
    fn drop(&mut self) {
        self.background.set_visible(false);
    }
}

impl<'a, 'b> Construction<'a, 'b> {
    fn new(
        level: usize,
        background: &'a mut MapLoan<'b, RegularMap>,
        vram_manager: &mut VRamManager,
    ) -> Self {
        let game = GameState::new(level);
        game.load_level_background(background, vram_manager);
        background.commit(vram_manager);
        background.set_visible(true);
        Self { background, game }
    }

    fn update(
        mut self,
        input: &ButtonController,
        sfx: &mut Sfx,
        loader: &mut SpriteLoader,
    ) -> GamePhase<'a, 'b> {
        self.game.step(input, sfx);
        if input.is_just_pressed(Button::START) {
            self.game.force_place();
            GamePhase::Execute(Execute::new(self, sfx, loader))
        } else {
            GamePhase::Construction(self)
        }
    }

    fn render(&self, oam: &mut OamIterator, loader: &mut SpriteLoader) {
        self.game.render(loader, oam);
    }
}

impl<'a, 'b> Execute<'a, 'b> {
    fn new(construction: Construction<'a, 'b>, sfx: &mut Sfx, loader: &mut SpriteLoader) -> Self {
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
    ) -> GamePhase<'a, 'b> {
        if input.is_just_pressed(Button::START) {
            return GamePhase::Construction(self.construction);
        }

        match self.simulation.update(loader, sfx) {
            simulation::Outcome::Continue => GamePhase::Execute(self),
            simulation::Outcome::Loss => GamePhase::Construction(self.construction),
            simulation::Outcome::Win => GamePhase::NextLevel,
        }
    }

    fn render(&self, loader: &mut SpriteLoader, oam: &mut OamIterator) {
        self.simulation.render(oam);
        self.construction
            .game
            .render_arrows(loader, oam, Some(self.simulation.current_turn()));
    }
}

struct Execute<'a, 'b> {
    simulation: Simulation,
    construction: Construction<'a, 'b>,
}

#[derive(Default)]
enum GamePhase<'a, 'b> {
    #[default]
    Empty,
    Lament(Lament<'a, 'b>),
    Construction(Construction<'a, 'b>),
    Execute(Execute<'a, 'b>),
    NextLevel,
}

impl GamePhase<'_, '_> {
    fn update(
        &mut self,
        input: &ButtonController,
        sfx: &mut Sfx,
        loader: &mut SpriteLoader,
        vram_manger: &mut VRamManager,
    ) {
        *self = match core::mem::take(self) {
            GamePhase::Lament(lament) => lament.update(input, vram_manger),
            GamePhase::Construction(construction) => construction.update(input, sfx, loader),
            GamePhase::Execute(execute) => execute.update(input, sfx, loader),
            GamePhase::NextLevel => GamePhase::NextLevel,
            GamePhase::Empty => panic!("bad state"),
        }
    }

    fn render(&self, loader: &mut SpriteLoader, oam: &mut OamIterator) {
        match self {
            GamePhase::Empty => panic!("bad state"),
            GamePhase::Lament(lament) => lament.render(oam),
            GamePhase::Construction(construction) => construction.render(oam, loader),
            GamePhase::Execute(execute) => execute.render(loader, oam),
            GamePhase::NextLevel => {}
        }
    }
}

impl<'a, 'b> Game<'a, 'b> {
    pub fn new(level: usize, background: &'a mut MapLoan<'b, RegularMap>) -> Self {
        Self {
            phase: GamePhase::Lament(Lament::new(level, background)),
        }
    }

    pub fn update(
        &mut self,
        input: &ButtonController,
        sfx: &mut Sfx,
        loader: &mut SpriteLoader,
        vram_manager: &mut VRamManager,
    ) -> bool {
        self.phase.update(input, sfx, loader, vram_manager);
        matches!(self.phase, GamePhase::NextLevel)
    }

    pub fn render(&self, loader: &mut SpriteLoader, oam: &mut OamIterator) {
        self.phase.render(loader, oam)
    }

    pub fn set_background_visibility(&mut self, visible: bool) {
        match &mut self.phase {
            GamePhase::Construction(construction) => construction.background.set_visible(visible),
            GamePhase::Execute(execute) => execute.construction.background.set_visible(visible),
            _ => {}
        }
    }
}

pub struct Pausable<'a, 'b> {
    paused: Paused,
    menu: PauseMenu,
    game: Game<'a, 'b>,
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

    fn render(&self, oam: &mut OamIterator) {
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

impl<'a, 'b> Pausable<'a, 'b> {
    pub fn new(
        level: usize,
        maximum_level: usize,
        background: &'a mut MapLoan<'b, RegularMap>,
        loader: &mut SpriteLoader,
    ) -> Self {
        Self {
            paused: Paused::Playing,
            game: Game::new(level, background),
            menu: PauseMenu::new(loader, maximum_level, level),
        }
    }

    pub fn update(
        &mut self,
        input: &ButtonController,
        sfx: &mut Sfx,
        loader: &mut SpriteLoader,
        vram_manager: &mut VRamManager,
    ) -> Option<UpdateResult> {
        if input.is_just_pressed(Button::SELECT)
            || (matches!(self.paused, Paused::Paused) && input.is_just_pressed(Button::B))
        {
            self.paused = self.paused.change();
            match self.paused {
                Paused::Paused => self.game.set_background_visibility(false),
                Paused::Playing => self.game.set_background_visibility(true),
            }
        }

        if !matches!(self.paused, Paused::Paused) {
            if self.game.update(input, sfx, loader, vram_manager) {
                Some(UpdateResult::NextLevel)
            } else {
                None
            }
        } else {
            self.menu.update(input).map(UpdateResult::MenuSelection)
        }
    }

    pub fn render(&self, loader: &mut SpriteLoader, oam: &mut OamIterator) {
        if matches!(self.paused, Paused::Paused) {
            self.menu.render(oam);
        } else {
            self.game.render(loader, oam);
        }
    }
}
