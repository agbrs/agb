#![deny(clippy::indexing_slicing)]
#![deny(clippy::panicking_unwrap)]
#![deny(clippy::panic_in_result_fn)]

use core::ops::Neg;

use agb::fixnum::Vector2D;
use alloc::{boxed::Box, collections::VecDeque, vec::Vec};
use slotmap::{new_key_type, SlotMap};

use crate::{
    level::{self},
    map::{Map, MapElement},
    sfx::SoundEffect,
};

use super::animation::AnimationInstruction;

new_key_type! { pub struct EntityKey; }

#[derive(Default)]
pub struct EntityMap {
    map: SlotMap<EntityKey, Entity>,
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub enum Outcome {
    Continue,
    Loss,
    Win,
}

impl EntityMap {
    fn whats_at(&self, location: Vector2D<i32>) -> impl Iterator<Item = (EntityKey, &Entity)> {
        self.map
            .iter()
            .filter(move |(_, entity)| entity.location == location)
    }

    // returns whether killing this is a loss
    fn kill_entity(
        &mut self,
        entity: EntityKey,
        animations: &mut Vec<AnimationInstruction>,
    ) -> bool {
        if let Some((location, holding)) = self
            .map
            .get_mut(entity)
            .and_then(|x| x.take_holding().map(|y| (x.location, y)))
        {
            let new_key = self.map.insert(Entity {
                location,
                entity: holding,
            });

            animations.push(AnimationInstruction::Detatch(
                entity,
                new_key,
                self.map.get(new_key).and_then(|e| e.drop_effect()),
            ));
        }

        animations.push(AnimationInstruction::Die(
            entity,
            self.map.get(entity).and_then(|e| e.die_effect()),
        ));

        if let Some(entity) = self.map.remove(entity) {
            matches!(entity.entity, EntityType::Hero(_))
        } else {
            false
        }
    }

    pub fn add(
        &mut self,
        entity: crate::level::Item,
        location: Vector2D<i32>,
    ) -> AnimationInstruction {
        let idx = self.map.insert(Entity {
            location,
            entity: entity.into(),
        });

        AnimationInstruction::Add(idx, entity, location, None)
    }

    pub fn tick(&mut self, map: &Map, hero: Action) -> (Outcome, Vec<AnimationInstruction>) {
        let mut hero_has_died = false;
        let mut win_has_triggered = false;

        let mut animations = Vec::new();

        let mut entities_to_try_update = self
            .map
            .iter()
            .map(|(key, entity)| (key, entity.desired_action(map, self, hero)))
            .collect::<VecDeque<_>>();

        while let Some((entity_to_update_key, desired_action)) = entities_to_try_update.pop_front()
        {
            match desired_action {
                Action::Nothing => {}
                Action::Direction(direction) | Action::ChangeDirection(direction) => {
                    if matches!(desired_action, Action::ChangeDirection(_)) {
                        // first change the direction before processing the rest of the instructions
                        if let Some((Some(change), change_effect)) = self
                            .map
                            .get_mut(entity_to_update_key)
                            .map(|e| (e.change_direction(), e.change_effect()))
                        {
                            animations.push(AnimationInstruction::PriorityChange(
                                entity_to_update_key,
                                change,
                                change_effect,
                            ));
                        }
                    }

                    let Some(entity_to_update) = self.map.get(entity_to_update_key) else {
                        continue;
                    };

                    let entity_location = entity_to_update.location;

                    let desired_location = entity_location + direction.into();
                    let surface = map.get(desired_location);
                    if surface == MapElement::Wall {
                        animations.push(AnimationInstruction::FakeOutMove(
                            entity_to_update_key,
                            direction,
                            self.map
                                .get(entity_to_update_key)
                                .and_then(|e| e.fake_out_wall_effect()),
                        ));
                        continue;
                    }

                    let (can_move, explicit_stay_put, fake_out_effect) = {
                        let mut can_move = true;
                        let mut explicit_stay_put = false;
                        let mut fake_out_effect = None;

                        let move_attempt_resolutions: Vec<_> = self
                            .whats_at(desired_location)
                            .filter(|(k, _)| *k != entity_to_update_key)
                            .map(|(key, other_entity)| {
                                (key, resolve_move(entity_to_update, other_entity))
                            })
                            .collect();

                        for (other_entity_key, move_resolution) in move_attempt_resolutions {
                            match move_resolution {
                                MoveAttemptResolution::KillDie => {
                                    hero_has_died |=
                                        self.kill_entity(other_entity_key, &mut animations);
                                    hero_has_died |=
                                        self.kill_entity(entity_to_update_key, &mut animations);
                                    can_move = false;
                                }
                                MoveAttemptResolution::Kill => {
                                    hero_has_died |=
                                        self.kill_entity(other_entity_key, &mut animations);
                                    fake_out_effect = self
                                        .map
                                        .get(entity_to_update_key)
                                        .and_then(|x| x.kill_sound_effect());
                                    can_move = false;
                                }
                                MoveAttemptResolution::Die => {
                                    hero_has_died |=
                                        self.kill_entity(entity_to_update_key, &mut animations);
                                    can_move = false;
                                }
                                MoveAttemptResolution::CoExist => {}
                                MoveAttemptResolution::StayPut => {
                                    can_move = false;
                                    explicit_stay_put = true;
                                }
                                MoveAttemptResolution::AttemptPush => todo!(),
                            }
                        }

                        (can_move, explicit_stay_put, fake_out_effect)
                    };

                    if can_move {
                        if let Some(e) = self.map.get_mut(entity_to_update_key) {
                            e.location = desired_location;
                        }
                        let Some(entity_to_update) = self.map.get(entity_to_update_key) else {
                            continue;
                        };

                        animations.push(AnimationInstruction::Move(
                            entity_to_update_key,
                            desired_location,
                            entity_to_update.move_effect(),
                        ));

                        let overlap_resolutions: Vec<_> = self
                            .whats_at(desired_location)
                            .filter(|(k, _)| *k != entity_to_update_key)
                            .map(|(key, other_entity)| {
                                (key, resolve_overlap(entity_to_update, other_entity))
                            })
                            .collect();

                        for (other_entity_key, move_resolution) in overlap_resolutions {
                            match move_resolution {
                                OverlapResolution::Pickup => {
                                    animations.push(AnimationInstruction::Attach(
                                        entity_to_update_key,
                                        other_entity_key,
                                        self.map
                                            .get(other_entity_key)
                                            .and_then(|x| x.pickup_sound_effect()),
                                    ));
                                    let other = self.map.remove(other_entity_key).unwrap();

                                    if let Some((location, dropped)) =
                                        self.map.get_mut(entity_to_update_key).and_then(|x| {
                                            x.pickup(other.entity).map(|y| (x.location, y))
                                        })
                                    {
                                        let new_key = self.map.insert(Entity {
                                            location,
                                            entity: dropped,
                                        });

                                        animations.push(AnimationInstruction::Detatch(
                                            entity_to_update_key,
                                            new_key,
                                            self.map.get(new_key).and_then(|x| x.drop_effect()),
                                        ));
                                    }
                                }
                                OverlapResolution::CoExist => {}
                                OverlapResolution::Win => {
                                    win_has_triggered = true;
                                }
                                OverlapResolution::ToggleSystem(system) => {
                                    for (k, e) in self.map.iter_mut() {
                                        if let Some(change) = e.switch(system) {
                                            animations.push(AnimationInstruction::Change(
                                                k,
                                                change,
                                                e.change_effect(),
                                            ));
                                        }
                                    }
                                }
                                OverlapResolution::Die => {
                                    hero_has_died |=
                                        self.kill_entity(entity_to_update_key, &mut animations);
                                    break;
                                }
                                OverlapResolution::MoveAgain => {
                                    entities_to_try_update.push_front((
                                        entity_to_update_key,
                                        Action::Direction(direction),
                                    ));
                                }
                            }
                        }
                    } else {
                        animations.push(AnimationInstruction::FakeOutMove(
                            entity_to_update_key,
                            direction,
                            if explicit_stay_put {
                                self.map
                                    .get(entity_to_update_key)
                                    .and_then(|e| e.fake_out_wall_effect())
                            } else {
                                fake_out_effect.or_else(|| {
                                    self.map
                                        .get(entity_to_update_key)
                                        .and_then(|e| e.fake_out_effect())
                                })
                            },
                        ));
                    }
                }
            }
        }

        (
            if hero_has_died {
                Outcome::Loss
            } else if win_has_triggered {
                Outcome::Win
            } else {
                Outcome::Continue
            },
            animations,
        )
    }
}

enum MoveAttemptResolution {
    Kill,
    Die,
    KillDie,
    CoExist,
    StayPut,
    AttemptPush,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct SwitchSystem(usize);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum OverlapResolution {
    Pickup,
    CoExist,
    Win,
    ToggleSystem(SwitchSystem),
    Die,
    MoveAgain,
}

fn resolve_spikes(switable: &Switchable) -> OverlapResolution {
    if switable.active {
        OverlapResolution::Die
    } else {
        OverlapResolution::CoExist
    }
}

fn resolve_overlap(me: &Entity, other: &Entity) -> OverlapResolution {
    match (&me.entity, &other.entity) {
        (EntityType::Hero(_), EntityType::Stairs) => OverlapResolution::Win,
        (_, EntityType::Item(_)) => OverlapResolution::Pickup,
        (_, EntityType::Spikes(switch)) => resolve_spikes(switch),
        (_, EntityType::Switch(switch)) => OverlapResolution::ToggleSystem(switch.system),
        (_, EntityType::Enemy(_) | EntityType::Hero(_)) => OverlapResolution::Die,
        (_, EntityType::Ice) => OverlapResolution::MoveAgain,

        _ => OverlapResolution::CoExist,
    }
}

fn holding_attack_resolve(holding: Option<&EntityType>) -> MoveAttemptResolution {
    match holding {
        Some(&EntityType::Item(Item::Sword)) => MoveAttemptResolution::Kill,
        _ => MoveAttemptResolution::CoExist,
    }
}

fn squid_holding_attack_resolve(me: &Squid, other: &Entity) -> MoveAttemptResolution {
    match (me.holding.as_deref(), &other.entity, other.holding()) {
        (
            Some(&EntityType::Item(Item::Sword)),
            EntityType::Enemy(Enemy::Squid(squid)),
            Some(&EntityType::Item(Item::Sword)),
        ) => {
            if squid.direction == -me.direction {
                MoveAttemptResolution::KillDie
            } else {
                MoveAttemptResolution::Kill
            }
        }
        (Some(&EntityType::Item(Item::Sword)), EntityType::Enemy(_), None) => {
            MoveAttemptResolution::Kill
        }
        (_, EntityType::Enemy(Enemy::Squid(squid)), Some(&EntityType::Item(Item::Sword))) => {
            if squid.direction == -me.direction {
                MoveAttemptResolution::Die
            } else {
                MoveAttemptResolution::StayPut
            }
        }
        (_, EntityType::Enemy(_), _) => MoveAttemptResolution::StayPut,
        (_, EntityType::Hero(_), _) => MoveAttemptResolution::Kill,
        _ => MoveAttemptResolution::CoExist,
    }
}

fn holding_door_resolve(holding: Option<&EntityType>) -> MoveAttemptResolution {
    match holding {
        Some(&EntityType::Item(Item::Key)) => MoveAttemptResolution::Kill,
        _ => MoveAttemptResolution::StayPut,
    }
}

fn switch_door_resolve(door: &Switchable) -> MoveAttemptResolution {
    if door.active {
        MoveAttemptResolution::CoExist
    } else {
        MoveAttemptResolution::StayPut
    }
}

fn resolve_move(mover: &Entity, into: &Entity) -> MoveAttemptResolution {
    match (&mover.entity, &into.entity) {
        (EntityType::Hero(hero), EntityType::Hero(_) | EntityType::Enemy(_)) => {
            holding_attack_resolve(hero.holding.as_deref())
        }
        (EntityType::Hero(hero), EntityType::Door) => holding_door_resolve(hero.holding.as_deref()),
        (EntityType::Enemy(Enemy::Squid(squid)), EntityType::Hero(_) | EntityType::Enemy(_)) => {
            squid_holding_attack_resolve(squid, into)
        }
        (EntityType::Enemy(_), EntityType::Hero(_) | EntityType::Enemy(_)) => {
            MoveAttemptResolution::Kill
        }
        (_, EntityType::SwitchedDoor(door)) => switch_door_resolve(door),
        (EntityType::Enemy(Enemy::Squid(squid)), EntityType::Door) => {
            holding_door_resolve(squid.holding.as_deref())
        }
        (_, EntityType::Door) => MoveAttemptResolution::StayPut,
        (_, _) => MoveAttemptResolution::CoExist,
    }
}

#[derive(Debug)]
pub struct Hero {
    holding: Option<Box<EntityType>>,
}

pub struct Entity {
    location: Vector2D<i32>,
    entity: EntityType,
}

#[derive(Debug)]
pub struct Switchable {
    system: SwitchSystem,
    active: bool,
}

#[derive(Debug)]
pub enum EntityType {
    Hero(Hero),
    Item(Item),
    Enemy(Enemy),
    Stairs,
    Door,
    SwitchedDoor(Switchable),
    Switch(Switchable),
    Spikes(Switchable),
    Ice,
}

#[derive(Debug)]
pub struct Squid {
    direction: Direction,
    holding: Option<Box<EntityType>>,
}

#[derive(Debug)]
pub enum Enemy {
    Slime,
    Squid(Squid),
}

#[derive(Debug)]
pub enum Item {
    Sword,
    Key,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Neg for Direction {
    type Output = Direction;

    fn neg(self) -> Self::Output {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

impl From<Direction> for Vector2D<i32> {
    fn from(val: Direction) -> Self {
        (&val).into()
    }
}
impl From<&Direction> for Vector2D<i32> {
    fn from(val: &Direction) -> Self {
        match val {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
        .into()
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Action {
    Nothing,
    Direction(Direction),
    ChangeDirection(Direction),
}

impl Entity {
    fn desired_action(&self, walls: &Map, entities: &EntityMap, hero_action: Action) -> Action {
        match &self.entity {
            EntityType::Hero(_) => hero_action,
            EntityType::Enemy(Enemy::Squid(squid)) => {
                let desired_location = self.location + squid.direction.into();
                let wall = walls.get(desired_location);

                if matches!(wall, MapElement::Wall) {
                    Action::ChangeDirection(-squid.direction)
                } else {
                    let can_move = entities
                        .whats_at(desired_location)
                        .map(|(_, other_entity)| resolve_move(self, other_entity))
                        .filter(|resolution| matches!(resolution, MoveAttemptResolution::StayPut))
                        .count()
                        == 0;

                    if can_move {
                        Action::Direction(squid.direction)
                    } else {
                        Action::ChangeDirection(-squid.direction)
                    }
                }
            }
            _ => Action::Nothing,
        }
    }

    fn pickup(&mut self, item: EntityType) -> Option<EntityType> {
        let holding = match &mut self.entity {
            EntityType::Hero(hero) => &mut hero.holding,
            EntityType::Enemy(Enemy::Squid(squid)) => &mut squid.holding,
            _ => panic!("this entity can't pick up things"),
        };

        let existing = core::mem::replace(holding, Some(Box::new(item)));
        existing.map(|x| *x)
    }

    fn take_holding(&mut self) -> Option<EntityType> {
        match &mut self.entity {
            EntityType::Hero(hero) => hero.holding.take().map(|x| *x),
            EntityType::Enemy(Enemy::Squid(squid)) => squid.holding.take().map(|x| *x),
            _ => None,
        }
    }

    fn holding(&self) -> Option<&EntityType> {
        match &self.entity {
            EntityType::Hero(hero) => hero.holding.as_deref(),
            EntityType::Enemy(Enemy::Squid(squid)) => squid.holding.as_deref(),
            _ => None,
        }
    }

    fn die_effect(&self) -> Option<SoundEffect> {
        match &self.entity {
            EntityType::Hero(_) => Some(SoundEffect::HeroDie),
            EntityType::Door => Some(SoundEffect::DoorOpen),
            EntityType::Enemy(Enemy::Slime) => Some(SoundEffect::SlimeDie),
            EntityType::Enemy(Enemy::Squid(_)) => Some(SoundEffect::SquidDie),
            _ => None,
        }
    }

    fn drop_effect(&self) -> Option<SoundEffect> {
        match &self.entity {
            EntityType::Item(Item::Key) => Some(SoundEffect::KeyDrop),
            EntityType::Item(Item::Sword) => Some(SoundEffect::SwordDrop),
            _ => None,
        }
    }

    fn move_effect(&self) -> Option<SoundEffect> {
        None
    }

    fn kill_sound_effect(&self) -> Option<SoundEffect> {
        match self.holding() {
            Some(EntityType::Item(Item::Sword)) => Some(SoundEffect::SwordKill),
            _ => None,
        }
    }

    fn change_effect(&self) -> Option<SoundEffect> {
        match &self.entity {
            EntityType::Switch(_) => Some(SoundEffect::SwitchToggle),
            EntityType::SwitchedDoor(_) => Some(SoundEffect::SwitchedDoorToggle),
            EntityType::Spikes(_) => Some(SoundEffect::SpikesToggle),
            _ => None,
        }
    }

    fn fake_out_wall_effect(&self) -> Option<SoundEffect> {
        match &self.entity {
            EntityType::Hero(_) => Some(SoundEffect::WallHit),
            _ => None,
        }
    }

    fn pickup_sound_effect(&self) -> Option<SoundEffect> {
        match &self.entity {
            EntityType::Item(Item::Key) => Some(SoundEffect::KeyPickup),
            EntityType::Item(Item::Sword) => Some(SoundEffect::SwordPickup),
            _ => None,
        }
    }

    fn fake_out_effect(&self) -> Option<SoundEffect> {
        None
    }

    fn change_direction(&mut self) -> Option<level::Item> {
        match &mut self.entity {
            EntityType::Enemy(Enemy::Squid(squid)) => {
                squid.direction = -squid.direction;

                if squid.direction == Direction::Up {
                    Some(level::Item::SquidUp)
                } else {
                    Some(level::Item::SquidDown)
                }
            }
            _ => None,
        }
    }

    fn switch(&mut self, system: SwitchSystem) -> Option<level::Item> {
        if let EntityType::SwitchedDoor(door) = &mut self.entity {
            if door.system == system {
                door.active = !door.active;
                return Some(if door.active {
                    level::Item::SwitchedOpenDoor
                } else {
                    level::Item::SwitchedClosedDoor
                });
            }
        }

        if let EntityType::Switch(switch) = &mut self.entity {
            if switch.system == system {
                switch.active = !switch.active;
                return Some(if switch.active {
                    level::Item::SwitchPressed
                } else {
                    level::Item::Switch
                });
            }
        }

        if let EntityType::Spikes(switch) = &mut self.entity {
            if switch.system == system {
                switch.active = !switch.active;
                return Some(if switch.active {
                    level::Item::SpikesUp
                } else {
                    level::Item::SpikesDown
                });
            }
        }

        None
    }
}

impl From<level::Entity> for Entity {
    fn from(value: level::Entity) -> Self {
        Entity {
            location: value.1,
            entity: value.0.into(),
        }
    }
}

impl From<level::Item> for EntityType {
    fn from(value: level::Item) -> Self {
        match value {
            level::Item::Hero => EntityType::Hero(Hero { holding: None }),
            level::Item::Slime => EntityType::Enemy(Enemy::Slime),
            level::Item::Stairs => EntityType::Stairs,
            level::Item::Sword => EntityType::Item(Item::Sword),
            level::Item::Door => EntityType::Door,
            level::Item::Key => EntityType::Item(Item::Key),
            level::Item::SwitchedOpenDoor => EntityType::SwitchedDoor(Switchable {
                system: SwitchSystem(0),
                active: true,
            }),
            level::Item::SwitchedClosedDoor => EntityType::SwitchedDoor(Switchable {
                system: SwitchSystem(0),
                active: false,
            }),
            level::Item::Switch => EntityType::Switch(Switchable {
                system: SwitchSystem(0),
                active: false,
            }),
            level::Item::SwitchPressed => EntityType::Switch(Switchable {
                system: SwitchSystem(0),
                active: true,
            }),
            level::Item::SpikesUp => EntityType::Spikes(Switchable {
                system: SwitchSystem(0),
                active: true,
            }),
            level::Item::SpikesDown => EntityType::Spikes(Switchable {
                system: SwitchSystem(0),
                active: false,
            }),
            level::Item::SquidUp => EntityType::Enemy(Enemy::Squid(Squid {
                direction: Direction::Up,
                holding: None,
            })),
            level::Item::SquidDown => EntityType::Enemy(Enemy::Squid(Squid {
                direction: Direction::Down,
                holding: None,
            })),
            level::Item::Ice => EntityType::Ice,
        }
    }
}
