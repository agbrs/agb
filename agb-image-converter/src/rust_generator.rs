use std::io;
use std::io::Write;

use crate::image_loader::Image;
use crate::palette16::Palette16OptimisationResults;
use crate::TileSize;

pub(crate) fn generate_code(
    output: &mut dyn Write,
    results: &Palette16OptimisationResults,
    image: &Image,
    tile_size: TileSize,
    crate_prefix: String,
) -> io::Result<()> {
    writeln!(output, "#[allow(non_upper_case_globals)]")?;
    writeln!(output, "pub const {}: {}::display::tile_data::TileData = {{", "test", crate_prefix)?;

    writeln!(
        output,
        "const PALETTE_DATA: &[{}::display::palette16::Palette16] = &[",
        crate_prefix,
    )?;

    for palette in &results.optimised_palettes {
        write!(
            output,
            "    {}::display::palette16::Palette16::new([",
            crate_prefix
        )?;

        for colour in palette.clone() {
            write!(output, "0x{:08x}, ", colour.to_rgb15())?;
        }

        for _ in palette.clone().into_iter().len()..16 {
            write!(output, "0x00000000, ")?;
        }

        writeln!(output, "]),")?;
    }

    writeln!(output, "];")?;
    writeln!(output)?;

    writeln!(output, "const TILE_DATA: &[u32] = &[",)?;

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
            )?;

            for inner_y in 0..tile_size / 8 {
                write!(output, "    ")?;

                for inner_x in 0..tile_size / 8 {
                    for j in inner_y * 8..inner_y * 8 + 8 {
                        write!(output, "0x")?;

                        for i in (inner_x * 8..inner_x * 8 + 8).rev() {
                            let colour = image.colour(x * tile_size + i, y * tile_size + j);
                            let colour_index = palette.colour_index(colour);

                            write!(output, "{:x}", colour_index)?;
                        }

                        write!(output, ", ")?;
                    }
                }
            }

            writeln!(output)?;
        }
    }

    writeln!(output, "];")?;
    writeln!(output)?;

    write!(output, "const PALETTE_ASSIGNMENT: &[u8] = &[")?;

    for (i, assignment) in results.assignments.iter().enumerate() {
        if i % 16 == 0 {
            write!(output, "\n    ")?;
        }
        write!(output, "{}, ", assignment)?;
    }

    writeln!(output, "\n];")?;

    writeln!(output, "{}::display::tile_data::TileData::new(PALETTE_DATA, TILE_DATA, PALETTE_ASSIGNMENT)\n}};", crate_prefix)?;

    Ok(())
}
