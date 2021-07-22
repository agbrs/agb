use proc_macro::TokenStream;
use syn::parse_macro_input;

use std::path::Path;

use quote::{quote, format_ident};

mod colour;
mod config;
mod image_loader;
mod palette16;
mod rust_generator;

use image_loader::Image;

use colour::Colour;

#[derive(Debug, Clone, Copy)]
pub(crate) enum TileSize {
    Tile8,
    Tile16,
}

impl TileSize {
    fn to_size(self) -> usize {
        match self {
            TileSize::Tile8 => 8,
            TileSize::Tile16 => 16,
        }
    }
}

#[proc_macro]
pub fn include_gfx(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::LitStr);

    let filename = input.value();

    let root = std::env::var("CARGO_MANIFEST_DIR").expect("Failed to get cargo manifest dir");
    let path = Path::new(&root).join(&*filename);
    let parent = path
        .parent()
        .expect("Expected a parent directory for the path");

    let config = config::parse(&path.to_string_lossy());

    let module_name = format_ident!("{}", path.file_stem().expect("Expected a file stem").to_string_lossy());
    let include_path = path.to_string_lossy();

    let images = config.images();
    let image_code = images
        .iter()
        .map(|(image_name, &image)| convert_image(image, parent, &image_name, &config.crate_prefix()).parse::<proc_macro2::TokenStream>().unwrap());

    let module = quote! {
        pub mod #module_name {
            const _: &[u8] = include_bytes!(#include_path);

            #(#image_code)*
        }
    };

    TokenStream::from(module)
}

fn convert_image(
    settings: &dyn config::Image,
    parent: &Path,
    variable_name: &str,
    crate_prefix: &str,
) -> String {
    let image_filename = &parent.join(&settings.filename());
    let image = Image::load_from_file(image_filename);

    let tile_size = settings.tilesize().to_size();
    if image.width % tile_size != 0 || image.height % tile_size != 0 {
        panic!("Image size not a multiple of tile size");
    }

    let optimiser = optimiser_for_image(&image, tile_size);
    let optimisation_results = optimiser.optimise_palettes(settings.transparent_colour());

    let mut writer = String::new();

    rust_generator::generate_code(
        &mut writer,
        variable_name,
        &optimisation_results,
        &image,
        &image_filename.to_string_lossy(),
        settings.tilesize(),
        crate_prefix.to_owned(),
    );

    writer
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
