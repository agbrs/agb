use agb::{
    display::GraphicsFrame,
    fixnum::{Num, Vector2D},
};
use alloc::vec::Vec;

use crate::{
    level::{Item, Level},
    sfx::Sfx,
};

use self::{
    animation::{Animation, RenderCache},
    entity::{Action, EntityMap},
};

mod animation;
mod entity;

pub use entity::Direction;
pub use entity::Outcome;

pub struct Simulation {
    entities: EntityMap,
    animation: Animation,
    level: &'static Level,
    move_idx: usize,
    outcome: Outcome,
    frame: usize,

    render_cache: Vec<RenderCache>,
}

impl Simulation {
    pub fn generate(
        a: impl Iterator<Item = (Item, Vector2D<i32>)>,
        level: &'static Level,
        sfx: &mut Sfx,
    ) -> Simulation {
        let mut entities = EntityMap::default();
        let mut animation = Animation::default();

        for (item, location) in a {
            animation.populate(entities.add(item, location), sfx);
        }

        let mut simulation = Simulation {
            entities,
            animation,
            move_idx: 0,
            level,
            outcome: Outcome::Continue,
            frame: 0,
            render_cache: Vec::new(),
        };

        simulation.cache_render();

        simulation
    }

    pub fn current_turn(&self) -> usize {
        self.move_idx.saturating_sub(1)
    }

    pub fn render(&self, frame: &mut GraphicsFrame) {
        for item in self.render_cache.iter() {
            item.render(frame);
        }
    }

    pub fn cache_render(&mut self) {
        self.render_cache = self.animation.cache_render(self.frame / 16);
        self.render_cache
            .sort_unstable_by_key(|x| x.sorting_number());
    }

    pub fn update(&mut self, sfx: &mut Sfx) -> Outcome {
        self.animation.increase_progress(Num::new(1) / 16);

        self.frame = self.frame.wrapping_add(1);

        let animation_result = self.animation.update(sfx);

        self.cache_render();

        if animation_result {
            if self.outcome != Outcome::Continue {
                return self.outcome;
            }
            let hero_move = self.level.directions.get(self.move_idx);
            if let Some(&hero_move) = hero_move {
                let (outcome, animation) = self
                    .entities
                    .tick(&self.level.map, Action::Direction(hero_move));
                self.move_idx += 1;
                self.outcome = outcome;
                for anim in animation {
                    self.animation.populate(anim, sfx);
                }
            } else {
                return Outcome::Loss;
            }
        }

        Outcome::Continue
    }
}
