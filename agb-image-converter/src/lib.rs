use palette16::Palette16OptimisationResults;
use proc_macro::TokenStream;
use syn::parse_macro_input;

use std::{
    fs::File,
    iter,
    path::{Path, PathBuf},
    process::Command,
    str,
};

use quote::{format_ident, quote};

mod aseprite;
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
    Tile32,
}

impl TileSize {
    fn to_size(self) -> usize {
        match self {
            TileSize::Tile8 => 8,
            TileSize::Tile16 => 16,
            TileSize::Tile32 => 32,
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

    let module_name = format_ident!(
        "{}",
        path.file_stem()
            .expect("Expected a file stem")
            .to_string_lossy()
    );
    let include_path = path.to_string_lossy();

    let images = config.images();
    let image_code = images.iter().map(|(image_name, &image)| {
        convert_image(image, parent, image_name, &config.crate_prefix())
    });

    let module = quote! {
        mod #module_name {
            const _: &[u8] = include_bytes!(#include_path);

            #(#image_code)*
        }
    };

    TokenStream::from(module)
}

#[proc_macro]
pub fn include_aseprite_inner(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::LitStr);
    let filename = input.value();

    let root = std::env::var("CARGO_MANIFEST_DIR").expect("Failed to get cargo manifest dir");
    let path = Path::new(&root).join(&*filename);

    let out_dir = std::env::var("OUT_DIR").expect("Expected OUT_DIR");

    let output_filename = Path::new(&out_dir).join(&*filename);
    let image_output = output_filename.with_extension("png");
    let json_output = output_filename.with_extension("json");

    let command = Command::new("aseprite")
        .args([
            &PathBuf::from("-b"),
            &path,
            &"--sheet".into(),
            &image_output,
            &"--format".into(),
            &"json-array".into(),
            &"--data".into(),
            &json_output,
            &"--list-tags".into(),
        ])
        .output()
        .expect("Could not run aseprite");
    assert!(
        command.status.success(),
        "Aseprite did not complete successfully : {}",
        str::from_utf8(&*command.stdout).unwrap_or("Output contains invalid string")
    );

    let json: aseprite::Aseprite = serde_json::from_reader(
        File::open(&json_output).expect("The json output from aseprite could not be openned"),
    )
    .expect("The output from aseprite could not be decoded");

    // check that the size of the sprites are valid

    assert!(
        json.frames[0].frame.w == json.frames[0].frame.h
            && json.frames[0].frame.w.is_power_of_two()
            && json.frames[0].frame.w <= 32
    );

    let image = Image::load_from_file(image_output.as_path());

    let optimised_results =
        optimiser_for_image(&image, json.frames[0].frame.w as usize).optimise_palettes(None);

    let (palette_data, tile_data, assignments) =
        palete_tile_data(&optimised_results, json.frames[0].frame.w as usize, &image);

    let palette_data = palette_data.iter().map(|colours| {
        quote! {
            Palette16::new([
                #(#colours),*
            ])
        }
    });

    let mut pre = 0;
    let sprites = json
        .frames
        .iter()
        .zip(assignments.iter())
        .map(|(f, assignment)| {
            let start: usize = pre;
            let end: usize = pre + (f.frame.w as usize / 8) * (f.frame.h as usize / 8) * 32;
            let data = &tile_data[start..end];
            pre = end;
            let width = f.frame.w as usize;
            let height = f.frame.h as usize;
            quote! {
                Sprite::new(
                    &PALETTES[#assignment],
                    &[
                        #(#data),*
                    ],
                    Size::from_width_height(#width, #height)
                )
            }
        });

    let tags = json.meta.frame_tags.iter().map(|tag| {
        let start = tag.from as usize;
        let end = tag.to as usize;
        let direction = tag.direction as usize;

        let name = &tag.name;
        assert!(start <= end, "Tag {} has start > end", name);

        quote! {
            #name => Tag::new(SPRITES, #start, #end, #direction)
        }
    });

    let include_path = path.to_string_lossy();

    let module = quote! {
        const _: &[u8] = include_bytes!(#include_path);


        const PALETTES: &[Palette16] = &[
            #(#palette_data),*
        ];

        pub const SPRITES: &[Sprite] = &[
            #(#sprites),*
        ];

        const TAGS: &TagMap = &TagMap::new(
            phf::phf_map! {
                #(#tags),*
            }
        );

    };

    TokenStream::from(module)
}

fn convert_image(
    settings: &dyn config::Image,
    parent: &Path,
    variable_name: &str,
    crate_prefix: &str,
) -> proc_macro2::TokenStream {
    let image_filename = &parent.join(&settings.filename());
    let image = Image::load_from_file(image_filename);

    let tile_size = settings.tilesize().to_size();
    if image.width % tile_size != 0 || image.height % tile_size != 0 {
        panic!("Image size not a multiple of tile size");
    }

    let optimiser = optimiser_for_image(&image, tile_size);
    let optimisation_results = optimiser.optimise_palettes(settings.transparent_colour());

    rust_generator::generate_code(
        variable_name,
        &optimisation_results,
        &image,
        &image_filename.to_string_lossy(),
        settings.tilesize(),
        crate_prefix.to_owned(),
    )
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

fn palete_tile_data(
    optimiser: &Palette16OptimisationResults,
    tile_size: usize,
    image: &Image,
) -> (Vec<Vec<u16>>, Vec<u8>, Vec<usize>) {
    let palette_data: Vec<Vec<u16>> = optimiser
        .optimised_palettes
        .iter()
        .map(|palette| {
            palette
                .clone()
                .into_iter()
                .map(|colour| colour.to_rgb15())
                .chain(iter::repeat(0))
                .take(16)
                .map(|colour| colour as u16)
                .collect()
        })
        .collect();

    let tiles_x = image.width / tile_size;
    let tiles_y = image.height / tile_size;

    let mut tile_data = vec![];

    for y in 0..tiles_y {
        for x in 0..tiles_x {
            let palette_index = optimiser.assignments[y * tiles_x + x];
            let palette = &optimiser.optimised_palettes[palette_index];

            for inner_y in 0..tile_size / 8 {
                for inner_x in 0..tile_size / 8 {
                    for j in inner_y * 8..inner_y * 8 + 8 {
                        for i in inner_x * 8..inner_x * 8 + 8 {
                            let colour = image.colour(x * tile_size + i, y * tile_size + j);
                            tile_data.push(palette.colour_index(colour));
                        }
                    }
                }
            }
        }
    }

    let tile_data = tile_data
        .chunks(2)
        .map(|chunk| chunk[0] | (chunk[1] << 4))
        .collect();

    let assignments = optimiser.assignments.clone();

    (palette_data, tile_data, assignments)
}
