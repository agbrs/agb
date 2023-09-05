#![deny(clippy::indexing_slicing)]
#![deny(clippy::panicking_unwrap)]
#![deny(clippy::panic_in_result_fn)]

use core::ops::Neg;

use agb::fixnum::Vector2D;
use alloc::{boxed::Box, vec::Vec};
use slotmap::{new_key_type, SlotMap};

use crate::{
    level::{self},
    map::{Map, MapElement},
    sfx::SoundEffect,
};

use super::animation::AnimationInstruction;

new_key_type! { pub struct EntityKey; }

pub struct EntityMap {
    map: SlotMap<EntityKey, Entity>,
}

pub struct EntityMapMaker {
    map: Vec<(crate::level::Item, Vector2D<i32>)>,
}

impl EntityMapMaker {
    pub fn new() -> Self {
        Self {
            map: Default::default(),
        }
    }

    pub fn add(&mut self, entity: crate::level::Item, location: Vector2D<i32>) {
        let idx = self.map.push((entity, location));
    }

    pub fn to_entity_map(mut self) -> (EntityMap, Vec<AnimationInstruction>) {
        self.map
            .sort_unstable_by_key(|(_, location)| location.x + location.y * 100);
        let mut entity_map = EntityMap {
            map: Default::default(),
        };
        let mut animations = Vec::new();
        for (entity, location) in self.map {
            animations.push(entity_map.add(entity, location));
        }
        (entity_map, animations)
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub enum Outcome {
    Continue,
    Loss,
    Win,
}

struct ActionResult {
    hero_has_died: bool,
    win_has_triggered: bool,
}

impl ActionResult {
    fn new(hero_has_died: bool, win_has_triggered: bool) -> Self {
        Self {
            hero_has_died,
            win_has_triggered,
        }
    }
}

struct HasMoved(bool);
struct WantsToMoveAgain(bool);

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

    // allow because while it's a lot of arguments, it's not confusing because they are all of different types
    #[allow(clippy::too_many_arguments)]
    fn attempt_move_in_direction(
        &mut self,
        map: &Map,
        animations: &mut Vec<AnimationInstruction>,
        entity_to_update_key: EntityKey,
        direction: Direction,
        can_turn_around: bool,
        push_depth: i32,
        entities_that_have_moved: &mut Vec<(EntityKey, Direction)>,
    ) -> (HasMoved, ActionResult) {
        let mut hero_has_died = false;
        let mut win_has_triggered = false;

        let Some(entity_to_update) = self.map.get(entity_to_update_key) else {
            return (
                HasMoved(false),
                ActionResult::new(hero_has_died, win_has_triggered),
            );
        };

        let entity_location = entity_to_update.location;

        let desired_location = entity_location + direction.into();
        let surface = map.get(desired_location);

        let mut should_die_later = false;

        let (can_move, explicit_stay_put, fake_out_effect) = if surface == MapElement::Wall {
            (false, true, None)
        } else {
            let mut can_move = true;
            let mut explicit_stay_put = false;
            let mut fake_out_effect = None;

            let move_attempt_resolutions: Vec<_> = self
                .whats_at(desired_location)
                .filter(|(k, _)| *k != entity_to_update_key)
                .map(|(key, other_entity)| {
                    (key, resolve_move(entity_to_update, other_entity, direction))
                })
                .collect();

            for (other_entity_key, move_resolution) in move_attempt_resolutions {
                match move_resolution {
                    MoveAttemptResolution::KillDie => {
                        hero_has_died |= self.kill_entity(other_entity_key, animations);
                        hero_has_died |= self.kill_entity(entity_to_update_key, animations);
                        can_move = false;
                    }
                    MoveAttemptResolution::Kill => {
                        hero_has_died |= self.kill_entity(other_entity_key, animations);
                        fake_out_effect = self
                            .map
                            .get(entity_to_update_key)
                            .and_then(|x| x.kill_sound_effect());
                        can_move = false;
                    }
                    MoveAttemptResolution::Die => {
                        hero_has_died |= self.kill_entity(entity_to_update_key, animations);
                        can_move = false;
                    }
                    MoveAttemptResolution::CoExist => {}
                    MoveAttemptResolution::StayPut => {
                        can_move = false;
                        explicit_stay_put = true;
                    }
                    MoveAttemptResolution::AttemptPush => {
                        let depth = push_depth - 1;
                        if depth >= 0 {
                            let (can_move_result, action_result) = self.attempt_move_in_direction(
                                map,
                                animations,
                                other_entity_key,
                                direction,
                                true,
                                depth,
                                entities_that_have_moved,
                            );

                            if !can_move_result.0 {
                                can_move = false;
                                explicit_stay_put = true;
                            }
                            hero_has_died |= action_result.hero_has_died;
                            win_has_triggered |= action_result.win_has_triggered;
                        } else {
                            can_move = false;
                            explicit_stay_put = true;
                        }
                    }
                    MoveAttemptResolution::DieLater => {
                        should_die_later = true;
                    }
                }
            }

            (can_move, explicit_stay_put, fake_out_effect)
        };

        if can_move {
            if let Some(e) = self.map.get_mut(entity_to_update_key) {
                e.location = desired_location;
            }
            entities_that_have_moved.push((entity_to_update_key, direction));

            let Some(entity_to_update) = self.map.get(entity_to_update_key) else {
                return (
                    HasMoved(can_move),
                    ActionResult::new(hero_has_died, win_has_triggered),
                );
            };

            let move_effect = entity_to_update.move_effect();

            animations.push(AnimationInstruction::Move(
                entity_to_update_key,
                desired_location,
                move_effect,
            ));
        } else if !should_die_later
            && explicit_stay_put
            && can_turn_around
            && self.map.get(entity_to_update_key).map(|e| e.turns_around()) == Some(true)
        {
            if let Some(directions_to_attempt) = self
                .map
                .get(entity_to_update_key)
                .and_then(|e| e.directions_to_attempt())
            {
                #[allow(clippy::indexing_slicing)]
                for &direction_to_attempt in directions_to_attempt {
                    let (can_move, action) = self.attempt_move_in_direction(
                        map,
                        animations,
                        entity_to_update_key,
                        direction_to_attempt,
                        false,
                        push_depth,
                        entities_that_have_moved,
                    );

                    if can_move.0 {
                        if let Some((Some(change), change_effect)) = self
                            .map
                            .get_mut(entity_to_update_key)
                            .map(|e| (e.change_direction(direction_to_attempt), e.change_effect()))
                        {
                            animations.push(AnimationInstruction::PriorityChange(
                                entity_to_update_key,
                                change,
                                change_effect,
                            ));
                        }

                        return (
                            can_move,
                            ActionResult::new(
                                hero_has_died || action.hero_has_died,
                                win_has_triggered || action.win_has_triggered,
                            ),
                        );
                    }
                }

                let last_direction_attempt = *directions_to_attempt.last().unwrap();

                animations.push(AnimationInstruction::FakeOutMove(
                    entity_to_update_key,
                    last_direction_attempt,
                    self.map
                        .get(entity_to_update_key)
                        .and_then(|e| e.fake_out_wall_effect()),
                ));

                if let Some((Some(change), change_effect)) =
                    self.map.get_mut(entity_to_update_key).map(|e| {
                        (
                            e.change_direction(last_direction_attempt),
                            e.change_effect(),
                        )
                    })
                {
                    animations.push(AnimationInstruction::PriorityChange(
                        entity_to_update_key,
                        change,
                        change_effect,
                    ));
                }

                return (
                    HasMoved(false),
                    ActionResult::new(hero_has_died, win_has_triggered),
                );
            }
        } else if can_turn_around {
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

        if should_die_later {
            hero_has_died |= self.kill_entity(entity_to_update_key, animations);
        }

        (
            HasMoved(can_move),
            ActionResult::new(hero_has_died, win_has_triggered),
        )
    }

    fn resolve_overlap_from_move(
        &mut self,
        animations: &mut Vec<AnimationInstruction>,
        entity_to_update_key: EntityKey,
    ) -> (WantsToMoveAgain, ActionResult) {
        let mut win_has_triggered = false;
        let mut hero_has_died = false;
        let mut should_move_again = false;

        let Some(entity_to_update) = self.map.get(entity_to_update_key) else {
            return (
                WantsToMoveAgain(should_move_again),
                ActionResult::new(hero_has_died, win_has_triggered),
            );
        };

        let location = entity_to_update.location;

        let overlap_resolutions: Vec<_> = self
            .whats_at(location)
            .filter(|(k, _)| *k != entity_to_update_key)
            .map(|(key, other_entity)| (key, resolve_overlap(entity_to_update, other_entity)))
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

                    if let Some((location, dropped)) = self
                        .map
                        .get_mut(entity_to_update_key)
                        .and_then(|x| x.pickup(other.entity).map(|y| (x.location, y)))
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
                    hero_has_died |= self.kill_entity(entity_to_update_key, animations);
                    break;
                }
                OverlapResolution::KillDie => {
                    hero_has_died |= self.kill_entity(other_entity_key, animations);
                    hero_has_died |= self.kill_entity(entity_to_update_key, animations);
                    break;
                }
                OverlapResolution::MoveAgain => {
                    should_move_again = true;
                }
                OverlapResolution::Teleport => {
                    // find other teleporter
                    let other_teleporter = self.map.iter().find(|(entity_key, entity)| {
                        *entity_key != other_entity_key
                            && matches!(entity.entity, EntityType::Teleporter)
                    });

                    if let Some((_other_teleporter_key, other_teleporter)) = other_teleporter {
                        let location_to_teleport_to = other_teleporter.location;
                        if self.whats_at(location_to_teleport_to).count() == 1 {
                            //ok, we can teleport
                            animations.push(AnimationInstruction::Move(
                                entity_to_update_key,
                                location_to_teleport_to,
                                Some(SoundEffect::TeleportEffect),
                            ));
                            if let Some(entity) = self.map.get_mut(entity_to_update_key) {
                                entity.location = location_to_teleport_to;
                            }
                        }
                    }
                }
            }
        }
        (
            WantsToMoveAgain(should_move_again),
            ActionResult::new(hero_has_died, win_has_triggered),
        )
    }

    pub fn tick(&mut self, map: &Map, hero: Action) -> (Outcome, Vec<AnimationInstruction>) {
        let mut hero_has_died = false;
        let mut win_has_triggered = false;

        let mut animations = Vec::new();

        let mut entities_to_try_update = self
            .map
            .iter()
            .map(|(key, entity)| (key, entity.desired_action(hero)))
            .filter_map(|(key, action)| match action {
                Action::Nothing => None,
                Action::Direction(direction) => Some((key, direction)),
            })
            .collect::<Vec<_>>();

        let mut first_loop = true;

        while !entities_to_try_update.is_empty() {
            let mut entities_that_have_moved = Vec::new();

            for (entity_to_update_key, direction) in entities_to_try_update.drain(..) {
                let (_, action_result) = self.attempt_move_in_direction(
                    map,
                    &mut animations,
                    entity_to_update_key,
                    direction,
                    first_loop,
                    self.map
                        .get(entity_to_update_key)
                        .and_then(|e| e.push_depth())
                        .unwrap_or(0),
                    &mut entities_that_have_moved,
                );

                hero_has_died |= action_result.hero_has_died;
                win_has_triggered |= action_result.win_has_triggered;
            }

            for (entity_to_update_key, direction) in entities_that_have_moved {
                let (should_move_again, action_result) =
                    self.resolve_overlap_from_move(&mut animations, entity_to_update_key);

                if should_move_again.0 {
                    entities_to_try_update.push((entity_to_update_key, direction));
                }

                hero_has_died |= action_result.hero_has_died;
                win_has_triggered |= action_result.win_has_triggered;
            }

            first_loop = false;
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
    DieLater,
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
    KillDie,
    MoveAgain,
    Teleport,
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
        (EntityType::Hero(_) | EntityType::Enemy(Enemy::Moving(_)), EntityType::Item(_)) => {
            OverlapResolution::Pickup
        }
        (EntityType::MovableBlock, EntityType::Spikes(_)) => OverlapResolution::CoExist,
        (_, EntityType::Spikes(switch)) => resolve_spikes(switch),
        (_, EntityType::Switch(switch)) => OverlapResolution::ToggleSystem(switch.system),
        (_, EntityType::Enemy(_)) => OverlapResolution::Die,
        (_, EntityType::Ice) => OverlapResolution::MoveAgain,
        (_, EntityType::Teleporter) => OverlapResolution::Teleport,
        (EntityType::MovableBlock, EntityType::Hole) => OverlapResolution::KillDie,
        (_, EntityType::Hole) => OverlapResolution::Die,

        _ => OverlapResolution::CoExist,
    }
}

fn holding_attack_resolve(
    holding: Option<&EntityType>,
    other: &Entity,
    direction: Direction,
) -> MoveAttemptResolution {
    match (holding, &other.entity) {
        (Some(&EntityType::Item(Item::Sword)), _) => MoveAttemptResolution::Kill,
        (_, EntityType::Enemy(Enemy::Moving(squid))) => {
            hero_walk_into_squid_interaction(squid, direction)
        }
        _ => MoveAttemptResolution::CoExist,
    }
}

fn squid_holding_attack_resolve(me: &Moving, other: &Entity) -> MoveAttemptResolution {
    match (me.holding.as_deref(), &other.entity, other.holding()) {
        (
            Some(&EntityType::Item(Item::Sword)),
            EntityType::Enemy(Enemy::Moving(squid)),
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
        (_, EntityType::Enemy(Enemy::Moving(squid)), Some(&EntityType::Item(Item::Sword))) => {
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

fn hero_walk_into_squid_interaction(squid: &Moving, direction: Direction) -> MoveAttemptResolution {
    if direction == -squid.direction {
        MoveAttemptResolution::DieLater
    } else {
        MoveAttemptResolution::CoExist
    }
}

fn resolve_move(mover: &Entity, into: &Entity, direction: Direction) -> MoveAttemptResolution {
    match (&mover.entity, &into.entity) {
        (EntityType::Hero(hero), EntityType::Hero(_) | EntityType::Enemy(_)) => {
            holding_attack_resolve(hero.holding.as_deref(), into, direction)
        }
        (EntityType::Hero(hero), EntityType::Door) => holding_door_resolve(hero.holding.as_deref()),
        (EntityType::Enemy(Enemy::Moving(squid)), EntityType::Hero(_) | EntityType::Enemy(_)) => {
            squid_holding_attack_resolve(squid, into)
        }
        (EntityType::Enemy(_), EntityType::Hero(_) | EntityType::Enemy(_)) => {
            MoveAttemptResolution::Kill
        }
        (_, EntityType::SwitchedDoor(door)) => switch_door_resolve(door),
        (EntityType::Enemy(Enemy::Moving(squid)), EntityType::Door) => {
            holding_door_resolve(squid.holding.as_deref())
        }
        (_, EntityType::Door) => MoveAttemptResolution::StayPut,
        (_, EntityType::MovableBlock) => MoveAttemptResolution::AttemptPush,
        (EntityType::MovableBlock, EntityType::Hero(_) | EntityType::Enemy(_)) => {
            MoveAttemptResolution::StayPut
        }
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
    Hole,
    SwitchedDoor(Switchable),
    Switch(Switchable),
    Spikes(Switchable),
    Ice,
    MovableBlock,
    Teleporter,
}

#[derive(Debug)]
pub struct Moving {
    direction: Direction,
    holding: Option<Box<EntityType>>,
    movable_enemy_type: MovableEnemyType,
}

#[derive(Debug, PartialEq, Eq)]
enum MovableEnemyType {
    Squid,
    Rotator,
}

#[derive(Debug)]
pub enum Enemy {
    Slime,
    Moving(Moving),
}

#[derive(Debug)]
pub enum Item {
    Sword,
    Key,
    Glove,
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
}

impl Entity {
    fn desired_action(&self, hero_action: Action) -> Action {
        match &self.entity {
            EntityType::Hero(_) => hero_action,
            EntityType::Enemy(Enemy::Moving(squid)) => Action::Direction(squid.direction),
            _ => Action::Nothing,
        }
    }

    fn turns_around(&self) -> bool {
        matches!(self.entity, EntityType::Enemy(Enemy::Moving(_)))
    }

    fn pickup(&mut self, item: EntityType) -> Option<EntityType> {
        let holding = match &mut self.entity {
            EntityType::Hero(hero) => &mut hero.holding,
            EntityType::Enemy(Enemy::Moving(squid)) => &mut squid.holding,
            _ => panic!("this entity can't pick up things"),
        };

        let existing = core::mem::replace(holding, Some(Box::new(item)));
        existing.map(|x| *x)
    }

    fn take_holding(&mut self) -> Option<EntityType> {
        match &mut self.entity {
            EntityType::Hero(hero) => hero.holding.take().map(|x| *x),
            EntityType::Enemy(Enemy::Moving(squid)) => squid.holding.take().map(|x| *x),
            _ => None,
        }
    }

    fn push_depth(&self) -> Option<i32> {
        if matches!(self.holding(), Some(&EntityType::Item(Item::Glove))) {
            Some(i32::MAX)
        } else if matches!(
            self.entity,
            EntityType::Hero(_) | EntityType::Enemy(Enemy::Moving(_))
        ) {
            Some(1)
        } else {
            None
        }
    }

    fn holding(&self) -> Option<&EntityType> {
        match &self.entity {
            EntityType::Hero(hero) => hero.holding.as_deref(),
            EntityType::Enemy(Enemy::Moving(squid)) => squid.holding.as_deref(),
            _ => None,
        }
    }

    fn die_effect(&self) -> Option<SoundEffect> {
        match &self.entity {
            EntityType::Hero(_) => Some(SoundEffect::HeroDie),
            EntityType::Door => Some(SoundEffect::DoorOpen),
            EntityType::Enemy(Enemy::Slime) => Some(SoundEffect::SlimeDie),
            EntityType::Enemy(Enemy::Moving(e))
                if e.movable_enemy_type == MovableEnemyType::Squid =>
            {
                Some(SoundEffect::SquidDie)
            }
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

    fn directions_to_attempt(&self) -> Option<&'static [Direction]> {
        match &self.entity {
            EntityType::Enemy(Enemy::Moving(moving_type)) => {
                Some(match moving_type.movable_enemy_type {
                    MovableEnemyType::Squid => match moving_type.direction {
                        Direction::Up => &[Direction::Down],
                        Direction::Down => &[Direction::Up],
                        _ => panic!("Left and right movements are not valid for a squid"),
                    },
                    MovableEnemyType::Rotator => match moving_type.direction {
                        Direction::Up => &[Direction::Right, Direction::Left, Direction::Down],
                        Direction::Down => &[Direction::Left, Direction::Right, Direction::Up],
                        Direction::Left => &[Direction::Up, Direction::Down, Direction::Right],
                        Direction::Right => &[Direction::Down, Direction::Up, Direction::Left],
                    },
                })
            }
            _ => None,
        }
    }

    fn change_direction(&mut self, direction: Direction) -> Option<level::Item> {
        match &mut self.entity {
            EntityType::Enemy(Enemy::Moving(moving)) => {
                moving.direction = direction;

                match moving.movable_enemy_type {
                    MovableEnemyType::Squid => {
                        if direction == Direction::Up {
                            Some(level::Item::SquidUp)
                        } else {
                            Some(level::Item::SquidDown)
                        }
                    }
                    MovableEnemyType::Rotator => Some(match direction {
                        Direction::Up => level::Item::RotatorUp,
                        Direction::Down => level::Item::RotatorDown,
                        Direction::Left => level::Item::RotatorLeft,
                        Direction::Right => level::Item::RotatorRight,
                    }),
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
            level::Item::SquidUp => EntityType::Enemy(Enemy::Moving(Moving {
                direction: Direction::Up,
                holding: None,
                movable_enemy_type: MovableEnemyType::Squid,
            })),
            level::Item::SquidDown => EntityType::Enemy(Enemy::Moving(Moving {
                direction: Direction::Down,
                holding: None,
                movable_enemy_type: MovableEnemyType::Squid,
            })),
            level::Item::Ice => EntityType::Ice,
            level::Item::MovableBlock => EntityType::MovableBlock,
            level::Item::Glove => EntityType::Item(Item::Glove),
            level::Item::Teleporter => EntityType::Teleporter,
            level::Item::Hole => EntityType::Hole,
            level::Item::RotatorRight => EntityType::Enemy(Enemy::Moving(Moving {
                direction: Direction::Right,
                holding: None,
                movable_enemy_type: MovableEnemyType::Rotator,
            })),
            level::Item::RotatorLeft => EntityType::Enemy(Enemy::Moving(Moving {
                direction: Direction::Left,
                holding: None,
                movable_enemy_type: MovableEnemyType::Rotator,
            })),
            level::Item::RotatorUp => EntityType::Enemy(Enemy::Moving(Moving {
                direction: Direction::Up,
                holding: None,
                movable_enemy_type: MovableEnemyType::Rotator,
            })),
            level::Item::RotatorDown => EntityType::Enemy(Enemy::Moving(Moving {
                direction: Direction::Down,
                holding: None,
                movable_enemy_type: MovableEnemyType::Rotator,
            })),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use agb::hash_map::HashMap;

    #[test_case]
    fn check_all_puzzle_solutions_work(_gba: &mut agb::Gba) {
        let number_of_levels = crate::level::Level::num_levels();
        let mut failed_levels = Vec::new();

        #[derive(Debug)]
        enum CompleteSimulationResult {
            Success,
            ExplicitLoss,
            InputSequenceOver,
            MismatchedItems(HashMap<crate::level::Item, ()>),
        }

        fn check_level_has_valid_items(level: usize) -> HashMap<crate::level::Item, ()> {
            let level = crate::level::Level::get_level(level);

            let mut given_items = HashMap::new();

            for &item in level.items.iter() {
                *given_items.entry(item).or_insert(0) += 1;
            }

            let mut solution_items = HashMap::new();

            for entity in level.solution.iter() {
                *solution_items.entry(entity.0).or_insert(0) += 1;
            }

            let mut mismatched = HashMap::new();

            for (&item, &count) in solution_items.iter() {
                if *given_items.entry(item).or_insert(0) < count {
                    mismatched.insert(item, ());
                }
            }

            mismatched
        }

        fn check_level_works(level: usize) -> CompleteSimulationResult {
            let level = crate::level::Level::get_level(level);

            let mut simulator = EntityMapMaker::new();
            for entity in level.entities {
                simulator.add(entity.0, entity.1);
            }
            for solution_entity in level.solution {
                simulator.add(solution_entity.0, solution_entity.1);
            }

            let (mut simulator, _) = simulator.to_entity_map();

            for &direction in level.directions {
                let (outcome, _) = simulator.tick(&level.map, Action::Direction(direction));
                match outcome {
                    Outcome::Continue => {}
                    Outcome::Loss => return CompleteSimulationResult::ExplicitLoss,
                    Outcome::Win => return CompleteSimulationResult::Success,
                }
            }

            CompleteSimulationResult::InputSequenceOver
        }

        for level_idx in 0..number_of_levels {
            let mismatched_items = check_level_has_valid_items(level_idx);
            if !mismatched_items.is_empty() {
                failed_levels.push((
                    level_idx,
                    CompleteSimulationResult::MismatchedItems(mismatched_items),
                ))
            }
            let outcome = check_level_works(level_idx);
            match outcome {
                CompleteSimulationResult::ExplicitLoss
                | CompleteSimulationResult::InputSequenceOver => {
                    failed_levels.push((level_idx, outcome))
                }
                _ => {}
            }
        }

        if !failed_levels.is_empty() {
            agb::println!("Levels that failed were:");
            for (level, outcome) in failed_levels {
                agb::println!(
                    "Level: {}, reason {:?}, lament: {}",
                    level,
                    outcome,
                    crate::level::Level::get_level(level).name
                );
            }

            panic!("Level check failed");
        }
    }
}
