use std::env;

use agb_image_converter::{convert_image, ImageConverterConfig, TileSize};

fn main() {
    let args: Vec<_> = env::args().collect();

    let file_path = &args[1];
    let output_path = &args[2];
    convert_image(
        ImageConverterConfig::builder()
            .tile_size(TileSize::Tile8)
            .input_image(file_path.into())
            .output_file(output_path.into())
            .build(),
    );
}
