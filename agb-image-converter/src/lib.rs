use palette16::{Palette16OptimisationResults, Palette16Optimiser};
use proc_macro::TokenStream;
use proc_macro2::Literal;
use syn::parse::Parser;
use syn::{parse_macro_input, punctuated::Punctuated, LitStr};
use syn::{Expr, ExprLit, Lit};

use std::collections::HashMap;
use std::path::PathBuf;
use std::{iter, path::Path, str};

use quote::{format_ident, quote, ToTokens};

mod aseprite;
mod colour;
mod config;
mod font_loader;
mod image_loader;
mod palette16;
mod rust_generator;

use image::GenericImageView;
use image_loader::Image;

use colour::Colour;

#[derive(Debug, Clone, Copy)]
pub(crate) enum TileSize {
    Tile8,
    Tile16,
    Tile32,
}

pub(crate) enum Colours {
    Colours16,
    Colours256,
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

    let mut optimiser = Palette16Optimiser::new(None);
    let mut assignment_offsets = HashMap::new();
    let mut assignment_offset = 0;

    for (name, settings) in images.iter() {
        let image_filename = &parent.join(&settings.filename());
        let image = Image::load_from_file(image_filename);

        let tile_size = settings.tilesize().to_size();
        if image.width % tile_size != 0 || image.height % tile_size != 0 {
            panic!("Image size not a multiple of tile size");
        }

        add_to_optimiser(
            &mut optimiser,
            &image,
            tile_size,
            settings.transparent_colour(),
        );

        let num_tiles = image.width * image.height / settings.tilesize().to_size().pow(2);
        assignment_offsets.insert(name, assignment_offset);
        assignment_offset += num_tiles;
    }

    let optimisation_results = optimiser.optimise_palettes();

    let mut image_code = vec![];

    for (image_name, &image) in images.iter() {
        image_code.push(convert_image(
            image,
            parent,
            image_name,
            &config.crate_prefix(),
            &optimisation_results,
            assignment_offsets[image_name],
        ));
    }

    let module = quote! {
        mod #module_name {
            const _: &[u8] = include_bytes!(#include_path);

            #(#image_code)*
        }
    };

    TokenStream::from(module)
}

use quote::TokenStreamExt;
struct ByteString<'a>(&'a [u8]);
impl ToTokens for ByteString<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append(Literal::byte_string(self.0));
    }
}

#[proc_macro]
pub fn include_aseprite_inner(input: TokenStream) -> TokenStream {
    let parser = Punctuated::<LitStr, syn::Token![,]>::parse_separated_nonempty;
    let parsed = match parser.parse(input) {
        Ok(e) => e,
        Err(e) => return e.to_compile_error().into(),
    };

    let transparent_colour = Colour::from_rgb(255, 0, 255, 0);

    let mut optimiser = palette16::Palette16Optimiser::new(Some(transparent_colour));
    let mut images = Vec::new();
    let mut tags = Vec::new();

    let root = std::env::var("CARGO_MANIFEST_DIR").expect("Failed to get cargo manifest dir");

    let filenames: Vec<PathBuf> = parsed
        .iter()
        .map(|s| s.value())
        .map(|s| Path::new(&root).join(&*s))
        .collect();

    for filename in filenames.iter() {
        let (frames, tag) = aseprite::generate_from_file(filename);

        tags.push((tag, images.len()));

        for frame in frames {
            let width = frame.width();
            let height = frame.height();
            assert!(
                valid_sprite_size(width, height),
                "File {} contains sprites with unrepresentable size {}x{}",
                filename.display(),
                width,
                height
            );

            let image = Image::load_from_dyn_image(frame);
            add_to_optimiser(&mut optimiser, &image, 8, Some(transparent_colour));
            images.push(image);
        }
    }

    let optimised_results = optimiser.optimise_palettes();

    let (palette_data, tile_data, assignments) = palette_tile_data(&optimised_results, &images);

    let palette_data = palette_data.iter().map(|colours| {
        quote! {
            Palette16::new([
                #(#colours),*
            ])
        }
    });

    let mut pre = 0;
    let sprites = images
        .iter()
        .zip(assignments.iter())
        .map(|(f, assignment)| {
            let start: usize = pre;
            let end: usize = pre + (f.width / 8) * (f.height / 8) * 32;
            let data = ByteString(&tile_data[start..end]);
            pre = end;
            let width = f.width;
            let height = f.height;
            quote! {
                Sprite::new(
                    &PALETTES[#assignment],
                    align_bytes!(u16, #data),
                    Size::from_width_height(#width, #height)
                )
            }
        });

    let tags = tags.iter().flat_map(|(tag, num_images)| {
        tag.iter().map(move |tag| {
            let start = tag.from_frame() as usize + num_images;
            let end = tag.to_frame() as usize + num_images;
            let direction = tag.animation_direction() as usize;

            let name = tag.name();
            assert!(start <= end, "Tag {} has start > end", name);

            quote! {
                (#name, Tag::new(SPRITES, #start, #end, #direction))
            }
        })
    });

    let include_paths = filenames.iter().map(|s| {
        let s = s.as_os_str().to_string_lossy();
        quote! {
            const _: &[u8] = include_bytes!(#s);
        }
    });

    let module = quote! {
        #(#include_paths)*


        const PALETTES: &[Palette16] = &[
            #(#palette_data),*
        ];

        pub const SPRITES: &[Sprite] = &[
            #(#sprites),*
        ];

        const TAGS: &TagMap = &TagMap::new(
            &[
                #(#tags),*
            ]
        );

    };

    TokenStream::from(module)
}

fn convert_image(
    settings: &dyn config::Image,
    parent: &Path,
    variable_name: &str,
    crate_prefix: &str,
    optimisation_results: &Palette16OptimisationResults,
    assignment_offset: usize,
) -> proc_macro2::TokenStream {
    let image_filename = &parent.join(&settings.filename());
    let image = Image::load_from_file(image_filename);

    rust_generator::generate_code(
        variable_name,
        optimisation_results,
        &image,
        &image_filename.to_string_lossy(),
        settings.tilesize(),
        crate_prefix.to_owned(),
        assignment_offset,
    )
}

fn add_to_optimiser(
    palette_optimiser: &mut palette16::Palette16Optimiser,
    image: &Image,
    tile_size: usize,
    transparent_colour: Option<Colour>,
) {
    let tiles_x = image.width / tile_size;
    let tiles_y = image.height / tile_size;

    for y in 0..tiles_y {
        for x in 0..tiles_x {
            let mut palette = palette16::Palette16::new();

            for j in 0..tile_size {
                for i in 0..tile_size {
                    let colour = image.colour(x * tile_size + i, y * tile_size + j);

                    palette.add_colour(match (colour.is_transparent(), transparent_colour) {
                        (true, Some(transparent_colour)) => transparent_colour,
                        _ => colour,
                    });
                }
            }

            palette_optimiser.add_palette(palette);
        }
    }
}

fn palette_tile_data(
    optimiser: &Palette16OptimisationResults,
    images: &[Image],
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

    let mut tile_data = Vec::new();
    let tile_size = TileSize::Tile8;

    for image in images {
        add_image_to_tile_data(&mut tile_data, image, tile_size, &optimiser, 0)
    }

    let tile_data = collapse_to_4bpp(&tile_data);

    let assignments = optimiser.assignments.clone();

    (palette_data, tile_data, assignments)
}

fn collapse_to_4bpp(tile_data: &[u8]) -> Vec<u8> {
    tile_data
        .chunks(2)
        .map(|chunk| chunk[0] | (chunk[1] << 4))
        .collect()
}

fn add_image_to_tile_data(
    tile_data: &mut Vec<u8>,
    image: &Image,
    tile_size: TileSize,
    optimiser: &Palette16OptimisationResults,
    assignment_offset: usize,
) {
    let tile_size = tile_size.to_size();
    let tiles_x = image.width / tile_size;
    let tiles_y = image.height / tile_size;

    for y in 0..tiles_y {
        for x in 0..tiles_x {
            let palette_index = optimiser.assignments[y * tiles_x + x + assignment_offset];
            let palette = &optimiser.optimised_palettes[palette_index];

            for inner_y in 0..tile_size / 8 {
                for inner_x in 0..tile_size / 8 {
                    for j in inner_y * 8..inner_y * 8 + 8 {
                        for i in inner_x * 8..inner_x * 8 + 8 {
                            let colour = image.colour(x * tile_size + i, y * tile_size + j);
                            tile_data
                                .push(palette.colour_index(colour, optimiser.transparent_colour));
                        }
                    }
                }
            }
        }
    }
}

fn flatten_group(expr: &Expr) -> &Expr {
    match expr {
        Expr::Group(group) => &group.expr,
        _ => expr,
    }
}

#[proc_macro]
pub fn include_font(input: TokenStream) -> TokenStream {
    let parser = Punctuated::<Expr, syn::Token![,]>::parse_separated_nonempty;
    let parsed = match parser.parse(input) {
        Ok(e) => e,
        Err(e) => return e.to_compile_error().into(),
    };

    let all_args: Vec<_> = parsed.into_iter().collect();
    if all_args.len() != 2 {
        panic!("Include_font requires 2 arguments, got {}", all_args.len());
    }

    let filename = match flatten_group(&all_args[0]) {
        Expr::Lit(ExprLit {
            lit: Lit::Str(str_lit),
            ..
        }) => str_lit.value(),
        _ => panic!("Expected literal string as first argument to include_font"),
    };

    let font_size = match flatten_group(&all_args[1]) {
        Expr::Lit(ExprLit {
            lit: Lit::Float(value),
            ..
        }) => value.base10_parse::<f32>().expect("Invalid float literal"),
        Expr::Lit(ExprLit {
            lit: Lit::Int(value),
            ..
        }) => value
            .base10_parse::<i32>()
            .expect("Invalid integer literal") as f32,
        _ => panic!("Expected literal float or integer as second argument to include_font"),
    };

    let root = std::env::var("CARGO_MANIFEST_DIR").expect("Failed to get cargo manifest dir");
    let path = Path::new(&root).join(&*filename);

    let file_content = std::fs::read(&path).expect("Failed to read ttf file");

    let rendered = font_loader::load_font(&file_content, font_size);

    let include_path = path.to_string_lossy();

    quote!({
        let _ = include_bytes!(#include_path);

        #rendered
    })
    .into()
}

#[cfg(test)]
mod tests {
    use asefile::AnimationDirection;

    #[test]
    // These directions defined in agb and have these values. This is important
    // when outputting code for agb. If more animation directions are added then
    // we will have to support them there.
    fn directions_to_agb() {
        assert_eq!(AnimationDirection::Forward as usize, 0);
        assert_eq!(AnimationDirection::Reverse as usize, 1);
        assert_eq!(AnimationDirection::PingPong as usize, 2);
    }
}

fn valid_sprite_size(width: u32, height: u32) -> bool {
    match (width, height) {
        (8, 8) => true,
        (16, 16) => true,
        (32, 32) => true,
        (64, 64) => true,
        (16, 8) => true,
        (32, 8) => true,
        (32, 16) => true,
        (64, 32) => true,
        (8, 16) => true,
        (8, 32) => true,
        (16, 32) => true,
        (32, 64) => true,
        (_, _) => false,
    }
}
