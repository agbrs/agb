const LEVELS: &[&str] = &[
    "1-1.json", "1-2.json", "1-3.json", "1-4.json", "1-5.json", "1-6.json", "1-7.json", "1-8.json",
    "2-4.json", "2-2.json", "2-1.json", "2-3.json",
];

fn main() {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR environment variable must be specified");

    tiled_export::export_tilemap(&out_dir).expect("Failed to export tilemap");
    for &level in LEVELS {
        tiled_export::export_level(&out_dir, level).expect("Failed to export level");
    }
}

mod tiled_export {
    use serde::Deserialize;
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::{BufReader, BufWriter, Write};

    const COLLISION_TILE: i32 = 1;
    const KILL_TILE: i32 = 2;
    const WIN_TILE: i32 = 4;

    pub fn export_tilemap(out_dir: &str) -> std::io::Result<()> {
        let filename = "map/tilemap.json";
        println!("cargo:rerun-if-changed={filename}");
        let file = File::open(filename)?;
        let reader = BufReader::new(file);

        let tilemap: TiledTilemap = serde_json::from_reader(reader)?;

        let output_file = File::create(format!("{out_dir}/tilemap.rs"))?;
        let mut writer = BufWriter::new(output_file);

        let tile_data: HashMap<_, _> = tilemap
            .tiles
            .iter()
            .map(|tile| {
                (
                    tile.id,
                    match tile.tile_type.as_str() {
                        "Collision" => COLLISION_TILE,
                        "Kill" => KILL_TILE,
                        "Win" => WIN_TILE,
                        _ => 0,
                    },
                )
            })
            .collect();

        let tile_info = (0..tilemap.tilecount)
            .map(|id| *tile_data.get(&id).unwrap_or(&0))
            .map(|tile_type| tile_type.to_string())
            .collect::<Vec<String>>()
            .join(", ");

        writeln!(
            &mut writer,
            "pub const COLLISION_TILE: i32 = {COLLISION_TILE};",
        )?;

        writeln!(&mut writer, "pub const KILL_TILE: i32 = {KILL_TILE};")?;
        writeln!(&mut writer, "pub const WIN_TILE: i32 = {WIN_TILE};")?;

        writeln!(&mut writer, "pub const TILE_DATA: &[u32] = &[{tile_info}];")?;

        Ok(())
    }

    pub fn export_level(out_dir: &str, level_file: &str) -> std::io::Result<()> {
        let filename = format!("map/{level_file}");
        println!("cargo:rerun-if-changed={filename}");
        let file = File::open(filename)?;
        let reader = BufReader::new(file);

        let level: TiledLevel = serde_json::from_reader(reader)?;

        let output_file = File::create(format!("{out_dir}/{level_file}.rs"))?;
        let mut writer = BufWriter::new(output_file);

        let layer_1 = level.layers[0]
            .data
            .as_ref()
            .expect("Expected first layer to be a tile layer")
            .iter()
            .map(|id| get_map_id(*id).to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let layer_2 = level.layers[1]
            .data
            .as_ref()
            .expect("Expected second layer to be a tile layer")
            .iter()
            .map(|id| get_map_id(*id).to_string())
            .collect::<Vec<_>>()
            .join(", ");

        writeln!(&mut writer, "const WIDTH: u32 = {};", level.width)?;
        writeln!(&mut writer, "const HEIGHT: u32 = {};", level.height)?;
        writeln!(&mut writer, "const TILEMAP: &[u16] = &[{layer_1}];")?;
        writeln!(&mut writer, "const BACKGROUND: &[u16] = &[{layer_2}];")?;

        let objects = level.layers[2]
            .objects
            .as_ref()
            .expect("Expected third layer to be an object layer")
            .iter()
            .map(|object| (&object.object_type, (object.x, object.y)));
        let mut snails = vec![];
        let mut slimes = vec![];
        let mut enemy_stops = vec![];
        let mut player_start = None;

        for (object_type, (x, y)) in objects {
            match object_type.as_str() {
                "Snail Spawn" => snails.push((x, y)),
                "Slime Spawn" => slimes.push((x, y)),
                "Player Start" => player_start = Some((x, y)),
                "Enemy Stop" => enemy_stops.push((x, y)),
                _ => panic!("Unknown object type {}", object_type),
            }
        }

        let player_start = player_start.expect("Need a start place for the player");

        let slimes_str = slimes
            .iter()
            .map(|slime| format!("({}, {})", slime.0, slime.1))
            .collect::<Vec<_>>()
            .join(", ");
        let snails_str = snails
            .iter()
            .map(|slime| format!("({}, {})", slime.0, slime.1))
            .collect::<Vec<_>>()
            .join(", ");
        let enemy_stop_str = enemy_stops
            .iter()
            .map(|enemy_stop| format!("({}, {})", enemy_stop.0, enemy_stop.1))
            .collect::<Vec<_>>()
            .join(", ");

        writeln!(
            &mut writer,
            "const SNAILS: &[(i32, i32)] = &[{snails_str}];",
        )?;
        writeln!(
            &mut writer,
            "const SLIMES: &[(i32, i32)] = &[{slimes_str}];",
        )?;
        writeln!(
            &mut writer,
            "const ENEMY_STOPS: &[(i32, i32)] = &[{enemy_stop_str}];",
        )?;
        writeln!(
            &mut writer,
            "const START_POS: (i32, i32) = ({}, {});",
            player_start.0, player_start.1
        )?;

        writeln!(
            &mut writer,
            r#"
            use crate::Level;
            use agb::fixnum::Vector2D;

            pub const fn get_level() -> Level {{
                Level {{
                    background: TILEMAP,
                    foreground: BACKGROUND,
                    dimensions: Vector2D {{x: WIDTH, y: HEIGHT}},
                    collision: crate::map_tiles::tilemap::TILE_DATA,
    
                    enemy_stops: ENEMY_STOPS,
                    slimes: SLIMES,
                    snails: SNAILS,
                    start_pos: START_POS,
                }}
            }}
            "#
        )?;

        Ok(())
    }

    fn get_map_id(id: i32) -> i32 {
        match id {
            0 => 10,
            i => i - 1,
        }
    }

    #[derive(Deserialize)]
    struct TiledLevel {
        layers: Vec<TiledLayer>,
        width: i32,
        height: i32,
    }

    #[derive(Deserialize)]
    struct TiledLayer {
        data: Option<Vec<i32>>,
        objects: Option<Vec<TiledObject>>,
    }

    #[derive(Deserialize)]
    struct TiledObject {
        #[serde(rename = "type")]
        object_type: String,
        x: i32,
        y: i32,
    }

    #[derive(Deserialize)]
    struct TiledTilemap {
        tiles: Vec<TiledTile>,
        tilecount: i32,
    }

    #[derive(Deserialize)]
    struct TiledTile {
        id: i32,
        #[serde(rename = "type")]
        tile_type: String,
    }
}
