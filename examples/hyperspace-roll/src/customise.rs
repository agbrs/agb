use agb::{
    display::{
        object::{OamManaged, Object},
        tiled::{RegularBackgroundSize, RegularBackgroundTiles, TileFormat},
        Priority, HEIGHT, WIDTH,
    },
    input::{Button, Tri},
};

use alloc::vec::Vec;

use crate::{
    background::load_description,
    graphics::{FACE_SPRITES, MODIFIED_BOX, SELECTED_BOX, SELECT_BOX},
    Agb, Die, Face, PlayerDice,
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

fn net_position_for_index(idx: usize) -> (u32, u32) {
    if idx == 4 {
        (1, 0)
    } else if idx == 5 {
        (1, 2)
    } else {
        (idx as u32, 1)
    }
}

fn screen_position_for_index(idx: usize) -> (u32, u32) {
    let (x, y) = net_position_for_index(idx);
    (x * 32 + 20, y * 32 + HEIGHT as u32 - 3 * 32)
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

fn create_dice_display<'a>(gfx: &'a OamManaged, dice: &'_ PlayerDice) -> Vec<Object<'a>> {
    let mut objects = Vec::new();
    for (idx, dice) in dice.dice.iter().enumerate() {
        let mut obj = gfx.object_sprite(FACE_SPRITES.sprite_for_face(dice.faces[1]));
        obj.set_x((idx as i32 * 32 - 24 / 2 + 20) as u16);
        obj.set_y(16 - 24 / 2);

        obj.show();

        objects.push(obj);
    }
    objects
}

fn create_net<'a>(gfx: &'a OamManaged, die: &'_ Die, modified: &[usize]) -> Vec<Object<'a>> {
    let mut objects = Vec::new();
    for (idx, &face) in die.faces.iter().enumerate() {
        let mut obj = gfx.object_sprite(FACE_SPRITES.sprite_for_face(face));
        let (x, y) = screen_position_for_index(idx);
        obj.set_x((x - 24 / 2) as u16);
        obj.set_y((y - 24 / 2) as u16);

        obj.show();

        objects.push(obj);
    }

    for &m in modified.iter().chain(core::iter::once(&3)) {
        let mut obj = gfx.object_sprite(MODIFIED_BOX);
        let (x, y) = screen_position_for_index(m);
        obj.set_x((x - 32 / 2) as u16);
        obj.set_y((y - 32 / 2) as u16);

        obj.show();

        objects.push(obj);
    }

    objects
}

fn upgrade_position(idx: usize) -> (u32, u32) {
    (
        (WIDTH - 80) as u32,
        (idx * 32 + HEIGHT as usize - 3 * 32) as u32,
    )
}

fn create_upgrade_objects<'a>(gfx: &'a OamManaged, upgrades: &[Face]) -> Vec<Object<'a>> {
    let mut objects = Vec::new();
    for (idx, &upgrade) in upgrades.iter().enumerate() {
        let mut obj = gfx.object_sprite(FACE_SPRITES.sprite_for_face(upgrade));
        let (x, y) = upgrade_position(idx);
        obj.set_x((x - 24 / 2) as u16);
        obj.set_y((y - 24 / 2) as u16);

        obj.show();

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

    let mut _net = create_net(&agb.obj, &player_dice.dice[0], &[]);
    let mut _dice = create_dice_display(&agb.obj, &player_dice);

    agb.sfx.frame();

    let mut upgrades = crate::level_generation::generate_upgrades(level, &mut || agb.sfx.frame());
    let mut _upgrade_objects = create_upgrade_objects(&agb.obj, &upgrades);

    let mut input = agb::input::ButtonController::new();

    let mut select_box = agb.obj.object_sprite(SELECT_BOX.sprite(0));

    select_box.show();

    let mut selected_dice = agb.obj.object_sprite(SELECTED_BOX);
    selected_dice.hide();
    let mut selected_face = agb.obj.object_sprite(SELECTED_BOX);
    selected_face.hide();
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
        let mut bg_iter = agb.tiled.iter();

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
                selected_dice.hide();
                let new_dice = (cursor.dice as isize + lr as isize)
                    .rem_euclid(player_dice.dice.len() as isize)
                    as usize;
                if new_dice != cursor.dice {
                    cursor.dice = new_dice;
                    _net = create_net(
                        &agb.obj,
                        &player_dice.dice[cursor.dice],
                        &modified
                            .iter()
                            .filter_map(|x| (x.dice == cursor.dice).then_some(x.face))
                            .collect::<Vec<usize>>(),
                    );
                }

                select_box.set_x((cursor.dice as i32 * 32 - 32 / 2 + 20) as u16);
                select_box.set_y(0);

                if input.is_just_pressed(Button::A) {
                    selected_dice.set_x((cursor.dice as i32 * 32 - 32 / 2 + 20) as u16);
                    selected_dice.set_y(0);
                    selected_dice.show();
                    state = CustomiseState::Face;
                    agb.sfx.select();
                }
            }
            CustomiseState::Face => {
                cursor.face = move_net_position_lr(cursor.face, lr);
                cursor.face = move_net_position_ud(cursor.face, ud);

                let (x, y) = screen_position_for_index(cursor.face);
                select_box.set_x((x - 32 / 2) as u16);
                select_box.set_y((y - 32 / 2) as u16);
                selected_face.hide();

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
                    selected_face.set_x((x - 32 / 2) as u16);
                    selected_face.set_y((y - 32 / 2) as u16);
                    selected_face.show();

                    cursor.upgrade += upgrades.len();

                    state = CustomiseState::Upgrade;
                    agb.sfx.select();
                }
            }
            CustomiseState::Upgrade => {
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
                select_box.set_x((x - 32 / 2) as u16);
                select_box.set_y((y - 32 / 2) as u16);

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
                    _upgrade_objects = create_upgrade_objects(&agb.obj, &upgrades);

                    _net = create_net(
                        &agb.obj,
                        &player_dice.dice[cursor.dice],
                        &modified
                            .iter()
                            .filter_map(|x| (x.dice == cursor.dice).then_some(x.face))
                            .collect::<Vec<usize>>(),
                    );
                    _dice = create_dice_display(&agb.obj, &player_dice);
                    state = CustomiseState::Face;
                    agb.sfx.accept();
                }
            }
        }

        if upgrades.is_empty() {
            break;
        }

        select_box.set_sprite(agb.obj.sprite(SELECT_BOX.animation_sprite(counter / 10)));

        agb.star_background.update();
        let _ = agb::rng::gen();
        agb.sfx.frame();
        agb.vblank.wait_for_vblank();
        agb.star_background.commit();
        agb.obj.commit();
        descriptions_map.commit();
        help_background.commit();

        help_background.show(&mut bg_iter);
        if description_map_visible {
            descriptions_map.show(&mut bg_iter);
        }

        agb.star_background.show(&mut bg_iter);

        bg_iter.commit();
    }

    player_dice
}
