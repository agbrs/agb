use crate::palette16::Palette16OptimisationResults;
use crate::TileSize;
use crate::{image_loader::Image, ByteString};

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use std::iter;

pub(crate) fn generate_code(
    output_variable_name: &str,
    results: &Palette16OptimisationResults,
    image: &Image,
    image_filename: &str,
    tile_size: TileSize,
    crate_prefix: String,
) -> TokenStream {
    let crate_prefix = format_ident!("{}", crate_prefix);
    let output_variable_name = format_ident!("{}", output_variable_name);

    let palette_data = results.optimised_palettes.iter().map(|palette| {
        let colours = palette
            .clone()
            .into_iter()
            .map(|colour| colour.to_rgb15())
            .chain(iter::repeat(0))
            .take(16)
            .map(|colour| colour as u16);

        quote! {
            #crate_prefix::display::palette16::Palette16::new([
                #(#colours),*
            ])
        }
    });

    let tile_size = tile_size.to_size();

    let tiles_x = image.width / tile_size;
    let tiles_y = image.height / tile_size;

    let mut tile_data = vec![];

    for y in 0..tiles_y {
        for x in 0..tiles_x {
            let palette_index = results.assignments[y * tiles_x + x];
            let palette = &results.optimised_palettes[palette_index];

            for inner_y in 0..tile_size / 8 {
                for inner_x in 0..tile_size / 8 {
                    for j in inner_y * 8..inner_y * 8 + 8 {
                        for i in inner_x * 8..inner_x * 8 + 8 {
                            let colour = image.colour(x * tile_size + i, y * tile_size + j);
                            tile_data
                                .push(palette.colour_index(colour, results.transparent_colour));
                        }
                    }
                }
            }
        }
    }

    let tile_data: Vec<_> = tile_data
        .chunks(2)
        .map(|chunk| (chunk[1] << 4) | chunk[0])
        .collect();

    let data = ByteString(&tile_data);

    let assignments = results.assignments.iter().map(|&x| x as u8);

    quote! {
        #[allow(non_upper_case_globals)]
        pub const #output_variable_name: #crate_prefix::display::tile_data::TileData = {
            const _: &[u8] = include_bytes!(#image_filename);

            const PALETTE_DATA: &[#crate_prefix::display::palette16::Palette16] = &[
                #(#palette_data),*
            ];

            const TILE_DATA: &[u8] = #data;

            const PALETTE_ASSIGNMENT: &[u8] = &[
                #(#assignments),*
            ];

            #crate_prefix::display::tile_data::TileData::new(PALETTE_DATA, TILE_DATA, PALETTE_ASSIGNMENT)
        };
    }
}
