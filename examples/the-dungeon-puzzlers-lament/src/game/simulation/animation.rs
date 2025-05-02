#![deny(clippy::indexing_slicing)]
#![deny(clippy::panicking_unwrap)]
#![deny(clippy::panic_in_result_fn)]

use core::ops::{Deref, DerefMut};

use agb::{
    display::{GraphicsFrame, object::Object},
    fixnum::{Num, Vector2D},
};
use alloc::vec::Vec;
use slotmap::SecondaryMap;

use crate::{
    level::Item,
    resources::HERO_CARRY,
    sfx::{Sfx, SoundEffect},
};

use super::entity::{Direction, EntityKey};

struct AnimationEntity {
    entity: Item,
    start_position: Vector2D<Num<i32, 10>>,
    rendered_position: Vector2D<Num<i32, 10>>,
    attached: Option<(Item, Num<i32, 10>)>,
}

#[derive(Default)]
struct ToPlay {
    moves: Vec<Move>,
    attach_progress: Vec<AttachProgress>,
    fakeout: Vec<FakeOutMove>,
    detatch: Vec<Detatch>,
    attach: Vec<Attach>,
    change: Vec<Change>,
    die: Vec<Die>,
}

fn convert_to_real_space(p: Vector2D<i32>) -> Vector2D<Num<i32, 10>> {
    p.change_base() * 16
}

impl ToPlay {
    pub fn populate(
        &mut self,
        instruction: AnimationInstruction,
        map: &mut SecondaryMap<EntityKey, AnimationEntity>,
        sfx: &mut Sfx<'_>,
    ) {
        match instruction {
            AnimationInstruction::Move(e, p, s) => {
                self.moves.push(Move(e, convert_to_real_space(p), s));
            }
            AnimationInstruction::FakeOutMove(e, d, s) => self.fakeout.push(FakeOutMove(e, d, s)),
            AnimationInstruction::Detatch(e, nk, s) => self.detatch.push(Detatch(e, nk, s)),
            AnimationInstruction::Attach(e, o, s) => {
                if let Some(entity_to_attach) = map.get(o) {
                    self.attach.push(Attach(e, entity_to_attach.entity, o, s))
                }
            }
            AnimationInstruction::Die(e, s) => self.die.push(Die(e, s)),
            AnimationInstruction::Add(e, item, p, s) => {
                map.insert(
                    e,
                    AnimationEntity {
                        entity: item,
                        start_position: convert_to_real_space(p),
                        rendered_position: convert_to_real_space(p),
                        attached: None,
                    },
                );
                sfx.play_sound_effect(s);
            }
            AnimationInstruction::Change(e, i, s) => self.change.push(Change(e, i, s)),
            AnimationInstruction::PriorityChange(e, i, s) => {
                if let Some(entity) = map.get_mut(e) {
                    entity.entity = i;
                    sfx.play_sound_effect(s);
                }
            }
        }
    }
}

#[derive(Default)]
pub struct Animation {
    map: Map,
    to_play: ToPlay,
    ease: Num<i32, 10>,
    time: Num<i32, 10>,
}

#[derive(Default)]
struct Map {
    map: SecondaryMap<EntityKey, AnimationEntity>,
}

fn attached_offset() -> Vector2D<Num<i32, 10>> {
    Vector2D::new(0, -10).change_base()
}

pub struct RenderCache {
    y: i32,
    item: Item,
    held: bool,
    object: Object,
}

impl RenderCache {
    pub fn render(&self, frame: &mut GraphicsFrame) {
        self.object.show(frame);
    }

    pub fn sorting_number(&self) -> i32 {
        let mut score = 0;
        if matches!(
            self.item,
            Item::Stairs | Item::Switch | Item::SwitchPressed | Item::SpikesDown | Item::SpikesUp
        ) {
            score += 100000;
        }

        if self.held {
            score -= 10000;
        }

        if matches!(self.item, Item::Hero) {
            score -= 1000;
        }

        score -= self.y;

        score
    }
}

impl Map {
    fn set_entity_start_location(
        &mut self,
        entity: EntityKey,
        destination: Vector2D<Num<i32, 10>>,
    ) {
        if let Some(entity) = self.map.get_mut(entity) {
            entity.rendered_position = destination;
            entity.start_position = destination;
        }
    }

    fn set_entity_to_start_location(&mut self, entity: EntityKey) {
        if let Some(entity) = self.map.get_mut(entity) {
            entity.rendered_position = entity.start_position;
        }
    }
}

impl Deref for Map {
    type Target = SecondaryMap<EntityKey, AnimationEntity>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl DerefMut for Map {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

impl Animation {
    pub fn populate(&mut self, instruction: AnimationInstruction, sfx: &mut Sfx) {
        self.to_play.populate(instruction, &mut self.map, sfx);
    }

    pub fn increase_progress(&mut self, amount_by: Num<i32, 10>) {
        self.time += amount_by;
        if self.time >= 1.into() {
            self.time = 1.into();
        }

        let ease_in = self.time;
        let sub = self.time - 1;
        let ease_out = -sub * sub + 1;
        self.ease = ease_in * (Num::new(1) - self.time) + ease_out * self.time;
    }

    pub fn cache_render(&self, animation_frame: usize) -> Vec<RenderCache> {
        let mut cache = Vec::new();

        for (_, entity) in self.map.iter() {
            if let Some((attached, attach_progress)) = entity.attached {
                let mut object = Object::new(attached.tag().animation_sprite(animation_frame));

                let pos = (entity.rendered_position + attached_offset() * attach_progress).floor()
                    + attached.map_entity_offset();
                object.set_pos(pos);

                cache.push(RenderCache {
                    object,
                    y: pos.y,
                    held: true,
                    item: attached,
                });
            }

            let sprite = if entity.entity == Item::Hero && entity.attached.is_some() {
                HERO_CARRY.animation_sprite(animation_frame)
            } else {
                entity.entity.shadow_tag().animation_sprite(animation_frame)
            };

            let mut object = Object::new(sprite);
            let position = entity.rendered_position.floor() + entity.entity.map_entity_offset();
            object.set_pos(position);

            cache.push(RenderCache {
                object,
                y: position.y,
                held: false,
                item: entity.entity,
            });
        }

        cache
    }

    pub fn update(&mut self, sfx: &mut Sfx) -> bool {
        if !self.to_play.moves.is_empty()
            || !self.to_play.fakeout.is_empty()
            || !self.to_play.attach_progress.is_empty()
        {
            if self.time >= 1.into() {
                // finalise animations
                for m in self.to_play.moves.drain(0..) {
                    let entity = m.0;
                    let destination = m.1;

                    self.map.set_entity_start_location(entity, destination);
                }

                for m in self.to_play.fakeout.drain(0..) {
                    let entity = m.0;

                    self.map.set_entity_to_start_location(entity);
                }

                for m in self.to_play.attach_progress.drain(0..) {
                    if let Some(ease) = self
                        .map
                        .get_mut(m.0)
                        .and_then(|x| x.attached.as_mut())
                        .map(|x| &mut x.1)
                    {
                        *ease = 1.into();
                    }
                }
            } else {
                // play moves and fakeouts
                for m in self.to_play.moves.iter_mut() {
                    let entity = m.0;
                    let destination = m.1;

                    sfx.play_sound_effect(m.2.take());

                    if let Some(entity) = self.map.get_mut(entity) {
                        let location = entity.start_position * (Num::<i32, 10>::new(1) - self.ease)
                            + destination * self.ease;

                        entity.rendered_position = location;
                    }
                }

                for m in self.to_play.fakeout.iter_mut() {
                    let entity = m.0;
                    let direction = m.1;
                    let direction = convert_to_real_space(direction.into());

                    sfx.play_sound_effect(m.2.take());

                    let go_to = direction / 2;

                    let start = (self.ease * 2 - 1).abs();
                    let end_multiplier = -start + 1;

                    if let Some(entity) = self.map.get_mut(entity) {
                        let location = entity.start_position + go_to * end_multiplier;

                        entity.rendered_position = location;
                    }
                }

                for m in self.to_play.attach_progress.iter_mut() {
                    sfx.play_sound_effect(m.1.take());
                    if let Some(ease) = self
                        .map
                        .get_mut(m.0)
                        .and_then(|x| x.attached.as_mut())
                        .map(|x| &mut x.1)
                    {
                        *ease = self.ease;
                    }
                }
            }
        } else if !self.to_play.detatch.is_empty() {
            self.time = 0.into();
            for detatch in self.to_play.detatch.drain(0..) {
                let entity = detatch.0;
                let new_key = detatch.1;

                sfx.play_sound_effect(detatch.2);

                if let Some((entity, attached)) = self
                    .map
                    .get_mut(entity)
                    .and_then(|x| x.attached.take().map(|y| (x, y)))
                {
                    let position = entity.start_position + attached_offset();
                    let destination_position = entity.start_position;
                    self.map.insert(
                        new_key,
                        AnimationEntity {
                            entity: attached.0,
                            start_position: position,
                            rendered_position: position,
                            attached: None,
                        },
                    );
                    self.to_play
                        .moves
                        .push(Move(new_key, destination_position, None));
                }
            }
        } else if !self.to_play.attach.is_empty() {
            self.time = 0.into();
            for attach in self.to_play.attach.drain(0..) {
                let entity_to_attach_to = attach.0;
                let other = attach.1;

                sfx.play_sound_effect(attach.3);

                if let Some(entity) = self.map.get_mut(entity_to_attach_to) {
                    entity.attached = Some((other, 0.into()));
                }

                self.map.remove(attach.2);
                self.to_play
                    .attach_progress
                    .push(AttachProgress(entity_to_attach_to, None));
            }
        } else if !self.to_play.change.is_empty() {
            self.time = 0.into();
            for change in self.to_play.change.drain(0..) {
                let entity = change.0;
                let item = change.1;

                sfx.play_sound_effect(change.2);

                if let Some(entity) = self.map.get_mut(entity) {
                    entity.entity = item;
                }
            }
        } else if !self.to_play.die.is_empty() {
            self.time = 0.into();
            for death in self.to_play.die.drain(0..) {
                sfx.play_sound_effect(death.1);

                let to_die = death.0;
                self.map.remove(to_die);
            }
        } else {
            self.time = 0.into();
            return true;
        }

        false
    }
}

struct Move(EntityKey, Vector2D<Num<i32, 10>>, Option<SoundEffect>);
struct FakeOutMove(EntityKey, Direction, Option<SoundEffect>);
struct Detatch(EntityKey, EntityKey, Option<SoundEffect>);
struct Attach(EntityKey, Item, EntityKey, Option<SoundEffect>);
struct AttachProgress(EntityKey, Option<SoundEffect>);
struct Die(EntityKey, Option<SoundEffect>);
struct Change(EntityKey, Item, Option<SoundEffect>);

#[derive(Clone, Debug)]
pub enum AnimationInstruction {
    Add(EntityKey, Item, Vector2D<i32>, Option<SoundEffect>),
    Move(EntityKey, Vector2D<i32>, Option<SoundEffect>),
    FakeOutMove(EntityKey, Direction, Option<SoundEffect>),
    Detatch(EntityKey, EntityKey, Option<SoundEffect>),
    Attach(EntityKey, EntityKey, Option<SoundEffect>),
    Change(EntityKey, Item, Option<SoundEffect>),
    PriorityChange(EntityKey, Item, Option<SoundEffect>),
    Die(EntityKey, Option<SoundEffect>),
}
