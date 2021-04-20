use std::path::PathBuf;

mod colour;
mod image_loader;
mod palette16;
mod rust_generator;

use image_loader::Image;

pub use colour::Colour;

#[derive(Debug, Clone, Copy)]
pub enum TileSize {
    Tile8,
    Tile16,
}

impl TileSize {
    fn to_size(&self) -> usize {
        match &self {
            TileSize::Tile8 => 8,
            TileSize::Tile16 => 16,
        }
    }
}

pub struct ImageConverterConfig {
    pub transparent_colour: Option<Colour>,
    pub tile_size: TileSize,
    pub input_image: PathBuf,
    pub output_file: PathBuf,
}

pub fn convert_image(settings: &ImageConverterConfig) {
    let image = Image::load_from_file(&settings.input_image);

    let tile_size = settings.tile_size.to_size();
    if image.width % tile_size != 0 || image.height % tile_size != 0 {
        panic!("Image size not a multiple of tile size");
    }

    let optimiser = optimiser_for_image(&image, tile_size);
    let optimisation_results = optimiser.optimise_palettes(settings.transparent_colour);

    let stdout = std::io::stdout();
    let handle = stdout.lock();
    let mut writer = std::io::BufWriter::new(handle);

    rust_generator::generate_code(
        &mut writer,
        &optimisation_results,
        &image,
        settings.tile_size,
        "HELLO",
    )
    .expect("Failed to write data");
}

fn optimiser_for_image(image: &Image, tile_size: usize) -> palette16::Palette16Optimiser {
    let tiles_x = image.width / tile_size;
    let tiles_y = image.height / tile_size;

    let mut palette_optimiser = palette16::Palette16Optimiser::new();

    for y in 0..tiles_y {
        for x in 0..tiles_x {
            let mut palette = palette16::Palette16::new();

            for j in 0..tile_size {
                for i in 0..tile_size {
                    let colour = image.colour(x * tile_size + i, y * tile_size + j);

                    palette.add_colour(colour);
                }
            }

            palette_optimiser.add_palette(palette);
        }
    }

    palette_optimiser
}
