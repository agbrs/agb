use std::fmt::Write;

use crate::image_loader::Image;
use crate::palette16::Palette16OptimisationResults;
use crate::TileSize;

pub(crate) fn generate_code(
    output: &mut dyn Write,
    output_variable_name: &str,
    results: &Palette16OptimisationResults,
    image: &Image,
    image_filename: &str,
    tile_size: TileSize,
    crate_prefix: String,
) {
    writeln!(output, "#[allow(non_upper_case_globals)]").unwrap();
    writeln!(output, "pub const {}: {}::display::tile_data::TileData = {{", output_variable_name, crate_prefix).unwrap();

    writeln!(output, "const _: &[u8] = include_bytes!(\"{}\");", image_filename).unwrap();

    writeln!(
        output,
        "const PALETTE_DATA: &[{}::display::palette16::Palette16] = &[",
        crate_prefix,
    ).unwrap();

    for palette in &results.optimised_palettes {
        write!(
            output,
            "    {}::display::palette16::Palette16::new([",
            crate_prefix
        ).unwrap();

        for colour in palette.clone() {
            write!(output, "0x{:08x}, ", colour.to_rgb15()).unwrap();
        }

        for _ in palette.clone().into_iter().len()..16 {
            write!(output, "0x00000000, ").unwrap();
        }

        writeln!(output, "]),").unwrap();
    }

    writeln!(output, "];").unwrap();
    writeln!(output).unwrap();

    writeln!(output, "const TILE_DATA: &[u32] = &[",).unwrap();

    let tile_size = tile_size.to_size();

    let tiles_x = image.width / tile_size;
    let tiles_y = image.height / tile_size;

    for y in 0..tiles_y {
        for x in 0..tiles_x {
            let palette_index = results.assignments[y * tiles_x + x];
            let palette = &results.optimised_palettes[palette_index];
            writeln!(
                output,
                "    /* {}, {} (palette index {}) */",
                x, y, palette_index
            ).unwrap();

            for inner_y in 0..tile_size / 8 {
                write!(output, "    ").unwrap();

                for inner_x in 0..tile_size / 8 {
                    for j in inner_y * 8..inner_y * 8 + 8 {
                        write!(output, "0x").unwrap();

                        for i in (inner_x * 8..inner_x * 8 + 8).rev() {
                            let colour = image.colour(x * tile_size + i, y * tile_size + j);
                            let colour_index = palette.colour_index(colour);

                            write!(output, "{:x}", colour_index).unwrap();
                        }

                        write!(output, ", ").unwrap();
                    }
                }
            }

            writeln!(output).unwrap();
        }
    }

    writeln!(output, "];").unwrap();
    writeln!(output).unwrap();

    write!(output, "const PALETTE_ASSIGNMENT: &[u8] = &[").unwrap();

    for (i, assignment) in results.assignments.iter().enumerate() {
        if i % 16 == 0 {
            write!(output, "\n    ").unwrap();
        }
        write!(output, "{}, ", assignment).unwrap();
    }

    writeln!(output, "\n];").unwrap();

    writeln!(output, "{}::display::tile_data::TileData::new(PALETTE_DATA, TILE_DATA, PALETTE_ASSIGNMENT)\n}};", crate_prefix).unwrap();
}
