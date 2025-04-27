use palette16::{Palette16OptimisationResults, Palette16Optimiser};
use palette256::Palette256;
use proc_macro::TokenStream;
use proc_macro2::Literal;
use syn::parse::{Parse, Parser};
use syn::{Expr, ExprLit, Lit, Token};
use syn::{parse_macro_input, punctuated::Punctuated};

use std::collections::HashMap;
use std::{path::Path, str};

use quote::{ToTokens, format_ident, quote};

mod aseprite;
mod colour;
mod config;
mod deduplicator;
mod font_loader;
mod image_loader;
mod palette16;
mod palette256;
mod rust_generator;

use image_loader::Image;

use colour::Colour;

mod sprite;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Colours {
    Colours16,
    Colours256,
}

struct BackgroundGfxOption {
    module_name: String,
    file_name: String,
    colours: Colours,
    deduplicate: bool,
}

impl config::Image for BackgroundGfxOption {
    fn filename(&self) -> String {
        self.file_name
            .clone()
            .replace(OUT_DIR_TOKEN, &get_out_dir(&self.file_name))
    }

    fn colours(&self) -> Colours {
        self.colours
    }

    fn deduplicate(&self) -> bool {
        self.deduplicate
    }
}

impl Parse for BackgroundGfxOption {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let module_name: syn::Ident = input.parse()?;
        let _: Token![=>] = input.parse()?;

        let lookahead = input.lookahead1();

        let colours = if lookahead.peek(syn::LitInt) {
            let num_colours: syn::LitInt = input.parse()?;

            match num_colours.base10_parse()? {
                16 => Colours::Colours16,
                256 => Colours::Colours256,
                _ => {
                    return Err(syn::Error::new_spanned(
                        num_colours,
                        "Number of colours must be 16 or 256",
                    ));
                }
            }
        } else {
            Colours::Colours16
        };

        let lookahead = input.lookahead1();

        let deduplicate = if lookahead.peek(syn::Ident) {
            let deduplicate: syn::Ident = input.parse()?;

            if deduplicate == "deduplicate" {
                true
            } else {
                return Err(syn::Error::new_spanned(
                    deduplicate,
                    "Must either be the literal deduplicate or missing",
                ));
            }
        } else {
            false
        };

        let file_name: syn::LitStr = input.parse()?;

        Ok(Self {
            module_name: module_name.to_string(),
            file_name: file_name.value(),
            colours,
            deduplicate,
        })
    }
}

struct IncludeBackgroundGfxInput {
    module_name: syn::Ident,
    visibility: syn::Visibility,
    crate_prefix: String,
    transparent_colour: Colour,
    background_gfx_options: Vec<BackgroundGfxOption>,
}

impl Parse for IncludeBackgroundGfxInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        let crate_prefix: syn::Ident = if lookahead.peek(Token![crate]) {
            let _: Token![crate] = input.parse()?;
            let _: Token![,] = input.parse()?;
            format_ident!("crate")
        } else {
            format_ident!("agb")
        };

        let visibility: syn::Visibility = input.parse()?;

        let _: Token![mod] = input.parse()?;
        let module_name: syn::Ident = input.parse()?;
        let _: Token![,] = input.parse()?;

        let lookahead = input.lookahead1();
        let transparent_colour: Colour = if lookahead.peek(syn::LitStr) {
            let colour_str: syn::LitStr = input.parse()?;
            let _: Token![,] = input.parse()?;
            colour_str
                .value()
                .parse()
                .map_err(|msg| syn::Error::new_spanned(colour_str, msg))?
        } else {
            Colour::from_rgb(255, 0, 255, 0)
        };

        let background_gfx_options =
            input.parse_terminated(BackgroundGfxOption::parse, Token![,])?;

        Ok(Self {
            module_name,
            visibility,
            crate_prefix: crate_prefix.to_string(),
            transparent_colour,
            background_gfx_options: background_gfx_options.into_iter().collect(),
        })
    }
}

impl config::Config for IncludeBackgroundGfxInput {
    fn crate_prefix(&self) -> String {
        self.crate_prefix.clone()
    }

    fn images(&self) -> HashMap<String, &dyn config::Image> {
        self.background_gfx_options
            .iter()
            .map(|options| (options.module_name.clone(), options as &dyn config::Image))
            .collect()
    }

    fn transparent_colour(&self) -> Option<Colour> {
        Some(self.transparent_colour)
    }
}

/// Including from the out directory is supported through the `$OUT_DIR` token.
///
/// ```rust,ignore
/// # #![no_std]
/// # #![no_main]
/// # use agb::include_background_gfx;
/// include_background_gfx!(mod generated_background, "000000", DATA => "$OUT_DIR/generated_background.aseprite");
/// ```
///
#[proc_macro]
pub fn include_background_gfx(input: TokenStream) -> TokenStream {
    let config = Box::new(parse_macro_input!(input as IncludeBackgroundGfxInput));

    let root = std::env::var("CARGO_MANIFEST_DIR").expect("Failed to get cargo manifest dir");

    let module_name = config.module_name.clone();
    let visibility = config.visibility.clone();
    include_gfx_from_config(config, visibility, module_name, Path::new(&root))
}

fn include_gfx_from_config(
    config: Box<dyn config::Config>,
    visibility: syn::Visibility,
    module_name: syn::Ident,
    parent: &Path,
) -> TokenStream {
    let images = config.images();

    let mut optimiser = Palette16Optimiser::new(config.transparent_colour());
    let mut assignment_offsets = HashMap::new();
    let mut assignment_offset = 0;

    let mut palette256 = Palette256::new();

    for (name, settings) in images.iter() {
        let image_filename = &parent.join(settings.filename());
        let image = Image::load_from_file(image_filename);

        match settings.colours() {
            Colours::Colours16 => {
                let tile_size = 8;
                if image.width % tile_size != 0 || image.height % tile_size != 0 {
                    panic!("Image size not a multiple of tile size");
                }

                add_to_optimiser(
                    &mut optimiser,
                    &image,
                    tile_size,
                    tile_size,
                    config.transparent_colour(),
                );

                let num_tiles = image.width * image.height / 8usize.pow(2);
                assignment_offsets.insert(name, assignment_offset);
                assignment_offset += num_tiles;
            }
            Colours::Colours256 => {
                palette256.add_image(&image);
            }
        }
    }

    let optimisation_results = optimiser
        .optimise_palettes()
        .expect("Failed to optimised palettes");
    let optimisation_results = palette256.extend_results(&optimisation_results);

    let mut image_code = vec![];

    for (image_name, &image) in images.iter() {
        let assignment_offset = match image.colours() {
            Colours::Colours16 => Some(assignment_offsets[image_name]),
            _ => None,
        };

        image_code.push(convert_image(
            image,
            parent,
            image_name,
            &config.crate_prefix(),
            &optimisation_results,
            assignment_offset,
        ));
    }

    let palette_code =
        rust_generator::generate_palette_code(&optimisation_results, &config.crate_prefix());

    let module = quote! {
        #visibility mod #module_name {
            #palette_code

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

struct IncludeColoursInput {
    module_name: syn::Path,
    filename: String,
}

impl Parse for IncludeColoursInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let module_name: syn::Path = input.parse()?;
        let _: Token![,] = input.parse()?;
        let filename: syn::LitStr = input.parse()?;

        Ok(Self {
            module_name,
            filename: filename.value(),
        })
    }
}

#[proc_macro]
pub fn include_colours_inner(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as IncludeColoursInput);
    let input_filename = input.filename;

    let module_name = input.module_name.clone();

    let root = std::env::var("CARGO_MANIFEST_DIR").expect("Failed to get cargo manifest dir");
    let input_filename = Path::new(&root).join(input_filename);

    let image = Image::load_from_file(Path::new(&input_filename));

    let mut palette_data = Vec::with_capacity(image.width * image.height);
    for y in 0..image.height {
        for x in 0..image.width {
            let rgb15 = image.colour(x, y).to_rgb15();
            palette_data.push(quote!(#module_name::display::Rgb15(#rgb15)));
        }
    }

    let filename = input_filename.to_string_lossy();

    TokenStream::from(quote! {
        {
            const _: &[u8] = include_bytes!(#filename);
            [#(#palette_data),*]
        }
    })
}

#[proc_macro]
pub fn include_aseprite_inner(input: TokenStream) -> TokenStream {
    sprite::include_regular(input)
}

#[proc_macro]
pub fn include_aseprite_256_inner(input: TokenStream) -> TokenStream {
    sprite::include_multi(input)
}

fn convert_image(
    settings: &dyn config::Image,
    parent: &Path,
    variable_name: &str,
    crate_prefix: &str,
    optimisation_results: &Palette16OptimisationResults,
    assignment_offset: Option<usize>,
) -> proc_macro2::TokenStream {
    let image_filename = &parent.join(settings.filename());
    let image = Image::load_from_file(image_filename);
    let deduplicate = settings.deduplicate();

    rust_generator::generate_code(
        variable_name,
        optimisation_results,
        &image,
        &image_filename.to_string_lossy(),
        crate_prefix.to_owned(),
        assignment_offset,
        deduplicate,
    )
}

fn add_to_optimiser(
    palette_optimiser: &mut palette16::Palette16Optimiser,
    image: &Image,
    tile_width: usize,
    tile_height: usize,
    transparent_colour: Option<Colour>,
) {
    let tiles_x = image.width / tile_width;
    let tiles_y = image.height / tile_height;

    for y in 0..tiles_y {
        for x in 0..tiles_x {
            let mut palette = palette16::Palette16::new();

            for j in 0..tile_height {
                for i in 0..tile_width {
                    let colour = image.colour(x * tile_width + i, y * tile_height + j);

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

fn collapse_to_4bpp(tile_data: &[u8]) -> Vec<u8> {
    tile_data
        .chunks(2)
        .map(|chunk| chunk[0] | (chunk[1] << 4))
        .collect()
}

fn add_image_to_tile_data(
    tile_data: &mut Vec<u8>,
    image: &Image,
    optimiser: &Palette16OptimisationResults,
    assignment_offset: usize,
    is_sprite: bool,
    remap_index: &[usize],
) {
    let tile_size = 8;
    let tiles_x = image.width / tile_size;
    let tiles_y = image.height / tile_size;

    for y in 0..tiles_y {
        for x in 0..tiles_x {
            let assignment = if is_sprite {
                assignment_offset
            } else {
                remap_index[y * tiles_x + x] + assignment_offset
            };

            let palette_index = optimiser.assignments[assignment];
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
}

fn add_image_256_to_tile_data(
    tile_data: &mut Vec<u8>,
    image: &Image,
    optimiser: &Palette16OptimisationResults,
) {
    let tile_size = 8;
    let tiles_x = image.width / tile_size;
    let tiles_y = image.height / tile_size;

    let all_colours: Vec<_> = optimiser
        .optimised_palettes
        .iter()
        .flat_map(|p| p.colours())
        .collect();

    for y in 0..tiles_y {
        for x in 0..tiles_x {
            for inner_y in 0..tile_size / 8 {
                for inner_x in 0..tile_size / 8 {
                    for j in inner_y * 8..inner_y * 8 + 8 {
                        for i in inner_x * 8..inner_x * 8 + 8 {
                            let colour = image.colour(x * tile_size + i, y * tile_size + j);
                            tile_data.push(all_colours.iter().position(|c| **c == colour).unwrap() as u8);
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

const OUT_DIR_TOKEN: &str = "$OUT_DIR";

fn get_out_dir(raw_input: &str) -> String {
    if raw_input.contains(OUT_DIR_TOKEN) {
        std::env::var("OUT_DIR").expect("Failed to get OUT_DIR")
    } else {
        String::new()
    }
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
