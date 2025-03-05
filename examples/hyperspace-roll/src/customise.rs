use agb::{
    display::{
        HEIGHT, Priority, WIDTH,
        object::Object,
        tiled::{RegularBackgroundSize, RegularBackgroundTiles, TileFormat},
    },
    input::{Button, Tri},
};

use alloc::vec::Vec;

use crate::{
    Agb, Die, Face, PlayerDice,
    background::load_description,
    graphics::{FACE_SPRITES, MODIFIED_BOX, SELECT_BOX, SELECTED_BOX},
};

enum CustomiseState {
    Dice,
    Face,
    Upgrade,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Cursor {
    dice: usize,
    face: usize,
    upgrade: usize,
}

fn net_position_for_index(idx: usize) -> (i32, i32) {
    if idx == 4 {
        (1, 0)
    } else if idx == 5 {
        (1, 2)
    } else {
        (idx as i32, 1)
    }
}

fn screen_position_for_index(idx: usize) -> (i32, i32) {
    let (x, y) = net_position_for_index(idx);
    (x * 32 + 20, y * 32 + HEIGHT - 3 * 32)
}

fn move_net_position_lr(idx: usize, direction: Tri) -> usize {
    match direction {
        Tri::Zero => idx,
        Tri::Positive => {
            if idx >= 4 {
                2
            } else {
                (idx + 1) % 3
            }
        }
        Tri::Negative => {
            if idx >= 4 {
                0
            } else {
                idx.checked_sub(1).unwrap_or(2)
            }
        }
    }
}

fn move_net_position_ud(idx: usize, direction: Tri) -> usize {
    match direction {
        Tri::Zero => idx,
        Tri::Negative => {
            if idx < 4 {
                4
            } else if idx == 4 {
                5
            } else if idx == 5 {
                1
            } else {
                unreachable!()
            }
        }
        Tri::Positive => {
            if idx < 4 {
                5
            } else if idx == 4 {
                1
            } else if idx == 5 {
                4
            } else {
                unreachable!()
            }
        }
    }
}

fn create_dice_display(dice: &PlayerDice) -> Vec<Object> {
    let mut objects = Vec::new();
    for (idx, dice) in dice.dice.iter().enumerate() {
        let mut obj = Object::new(FACE_SPRITES.sprite_for_face(dice.faces[1]));
        obj.set_position((idx as i32 * 32 - 24 / 2 + 20, 16 - 24 / 2));

        objects.push(obj);
    }
    objects
}

fn create_net(die: &Die, modified: &[usize]) -> Vec<Object> {
    let mut objects = Vec::new();
    for (idx, &face) in die.faces.iter().enumerate() {
        let mut obj = Object::new(FACE_SPRITES.sprite_for_face(face));
        let (x, y) = screen_position_for_index(idx);
        obj.set_position((x - 24 / 2, y - 24 / 2));

        objects.push(obj);
    }

    for &m in modified.iter().chain(core::iter::once(&3)) {
        let mut obj = Object::new(MODIFIED_BOX);
        let (x, y) = screen_position_for_index(m);
        obj.set_position((x - 32 / 2, y - 32 / 2));

        objects.push(obj);
    }

    objects
}

fn upgrade_position(idx: usize) -> (i32, i32) {
    (WIDTH - 80, idx as i32 * 32 + HEIGHT - 3 * 32)
}

fn create_upgrade_objects(upgrades: &[Face]) -> Vec<Object> {
    let mut objects = Vec::new();
    for (idx, &upgrade) in upgrades.iter().enumerate() {
        let mut obj = Object::new(FACE_SPRITES.sprite_for_face(upgrade));
        let (x, y) = upgrade_position(idx);
        obj.set_position((x - 24 / 2, y - 24 / 2));

        objects.push(obj);
    }
    objects
}

pub(crate) fn customise_screen(
    agb: &mut Agb,
    mut player_dice: PlayerDice,
    level: u32,
) -> PlayerDice {
    let mut descriptions_map = RegularBackgroundTiles::new(
        Priority::P1,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    let mut help_background = RegularBackgroundTiles::new(
        Priority::P1,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    agb.sfx.customise();
    agb.sfx.frame();
    descriptions_map.set_scroll_pos((-174i16, -52));

    help_background.set_scroll_pos((-148i16, -34));
    crate::background::load_help_text(&mut help_background, 0, (0, 0));

    // create the dice

    let mut net = create_net(&player_dice.dice[0], &[]);
    let mut dice = create_dice_display(&player_dice);

    agb.sfx.frame();

    let mut upgrades = crate::level_generation::generate_upgrades(level, &mut || agb.sfx.frame());
    let mut upgrade_objects = create_upgrade_objects(&upgrades);

    let mut input = agb::input::ButtonController::new();

    let mut select_box = Object::new(SELECT_BOX.sprite(0));

    let mut selected_dice = Object::new(SELECTED_BOX);
    let mut selected_face = Object::new(SELECTED_BOX);
    agb.sfx.frame();

    let mut counter = 0usize;

    let mut state = CustomiseState::Dice;

    let mut cursor = Cursor {
        dice: 0,
        face: 1,
        upgrade: 0,
    };

    let mut modified: Vec<Cursor> = Vec::new();

    let mut description_map_visible = true;

    loop {
        let mut frame = agb.gfx.frame();

        counter = counter.wrapping_add(1);
        input.update();
        let ud = (
            input.is_just_pressed(Button::UP),
            input.is_just_pressed(Button::DOWN),
        )
            .into();
        let lr = (
            input.is_just_pressed(Button::LEFT),
            input.is_just_pressed(Button::RIGHT),
        )
            .into();

        if ud != Tri::Zero || lr != Tri::Zero {
            agb.sfx.move_cursor();
        }

        match &mut state {
            CustomiseState::Dice => {
                let new_dice = (cursor.dice as isize + lr as isize)
                    .rem_euclid(player_dice.dice.len() as isize)
                    as usize;
                if new_dice != cursor.dice {
                    cursor.dice = new_dice;
                    net = create_net(
                        &player_dice.dice[cursor.dice],
                        &modified
                            .iter()
                            .filter_map(|x| (x.dice == cursor.dice).then_some(x.face))
                            .collect::<Vec<usize>>(),
                    );
                }

                select_box.set_position((cursor.dice as i32 * 32 - 32 / 2 + 20, 0));

                if input.is_just_pressed(Button::A) {
                    selected_dice.set_position((cursor.dice as i32 * 32 - 32 / 2 + 20, 0));
                    state = CustomiseState::Face;
                    agb.sfx.select();
                }
            }
            CustomiseState::Face => {
                selected_dice.show(&mut frame);

                cursor.face = move_net_position_lr(cursor.face, lr);
                cursor.face = move_net_position_ud(cursor.face, ud);

                let (x, y) = screen_position_for_index(cursor.face);
                select_box.set_position((x - 32 / 2, y - 32 / 2));

                if input.is_just_pressed(Button::B) {
                    state = CustomiseState::Dice;
                    agb.sfx.back();
                } else if input.is_just_pressed(Button::A)
                    && !upgrades.is_empty()
                    && !modified.contains(&Cursor {
                        dice: cursor.dice,
                        face: cursor.face,
                        upgrade: 0,
                    })
                {
                    selected_face.set_position((x - 32 / 2, y - 32 / 2));

                    cursor.upgrade += upgrades.len();

                    state = CustomiseState::Upgrade;
                    agb.sfx.select();
                }
            }
            CustomiseState::Upgrade => {
                selected_face.show(&mut frame);

                let old_upgrade = cursor.upgrade;
                cursor.upgrade = (cursor.upgrade as isize + ud as isize)
                    .rem_euclid(upgrades.len() as isize) as usize;

                if (upgrades[cursor.upgrade] as u32) < 17 {
                    if cursor.upgrade != old_upgrade {
                        load_description(upgrades[cursor.upgrade] as usize, &mut descriptions_map);
                    }

                    description_map_visible = true;
                } else {
                    description_map_visible = false;
                }

                let (x, y) = upgrade_position(cursor.upgrade);
                select_box.set_position((x - 32 / 2, y - 32 / 2));

                if input.is_just_pressed(Button::B) {
                    state = CustomiseState::Face;
                    agb.sfx.back();
                } else if input.is_just_pressed(Button::A)
                    && player_dice.dice[cursor.dice].faces[cursor.face] != upgrades[cursor.upgrade]
                {
                    description_map_visible = false;

                    modified.push(Cursor {
                        dice: cursor.dice,
                        face: cursor.face,
                        upgrade: 0,
                    });

                    player_dice.dice[cursor.dice].faces[cursor.face] = upgrades[cursor.upgrade];
                    upgrades.remove(cursor.upgrade);
                    upgrade_objects = create_upgrade_objects(&upgrades);

                    net = create_net(
                        &player_dice.dice[cursor.dice],
                        &modified
                            .iter()
                            .filter_map(|x| (x.dice == cursor.dice).then_some(x.face))
                            .collect::<Vec<usize>>(),
                    );
                    dice = create_dice_display(&player_dice);
                    state = CustomiseState::Face;
                    agb.sfx.accept();
                }
            }
        }

        for obj in net.iter().chain(dice.iter()).chain(upgrade_objects.iter()) {
            obj.show(&mut frame);
        }

        if upgrades.is_empty() {
            break;
        }

        select_box.set_sprite(SELECT_BOX.animation_sprite(counter / 10));

        select_box.show(&mut frame);

        agb.star_background.update();
        let _ = agb::rng::next_i32();
        agb.sfx.frame();
        agb.vblank.wait_for_vblank();

        help_background.show(&mut frame);
        if description_map_visible {
            descriptions_map.show(&mut frame);
        }

        agb.star_background.show(&mut frame);

        frame.commit();
    }

    player_dice
}
