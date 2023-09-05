use agb::{
    display::object::{OamIterator, SpriteLoader},
    fixnum::{Num, Vector2D},
};
use alloc::vec::Vec;

use crate::{
    level::{Item, Level},
    sfx::Sfx,
};

use self::{
    animation::{Animation, RenderCache},
    entity::{Action, EntityMap, EntityMapMaker},
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
        entities_to_add: impl Iterator<Item = (Item, Vector2D<i32>)>,
        level: &'static Level,
        sfx: &mut Sfx,
        loader: &mut SpriteLoader,
    ) -> Simulation {
        let mut entities = EntityMapMaker::new();
        let mut animation = Animation::default();

        for (item, location) in entities_to_add {
            entities.add(item, location);
        }

        let (entities, animations) = entities.to_entity_map();
        for ani in animations {
            animation.populate(ani, sfx);
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

        simulation.cache_render(loader);

        simulation
    }

    pub fn current_turn(&self) -> usize {
        self.move_idx.saturating_sub(1)
    }

    pub fn render(&self, oam: &mut OamIterator) {
        for item in self.render_cache.iter() {
            item.render(oam);
        }
    }

    pub fn cache_render(&mut self, sprite_loader: &mut SpriteLoader) {
        self.render_cache = self.animation.cache_render(sprite_loader, self.frame / 16);
        self.render_cache
            .sort_unstable_by_key(|x| x.sorting_number());
    }

    pub fn update(&mut self, sprite_loader: &mut SpriteLoader, sfx: &mut Sfx) -> Outcome {
        self.animation.increase_progress(Num::new(1) / 16);

        self.frame = self.frame.wrapping_add(1);

        let animation_result = self.animation.update(sfx);

        self.cache_render(sprite_loader);

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
