use crate::palette16::Palette16OptimisationResults;
use crate::{add_image_256_to_tile_data, add_image_to_tile_data, collapse_to_4bpp, TileSize};
use crate::{image_loader::Image, ByteString};

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use std::iter;

pub(crate) fn generate_palette_code(
    results: &Palette16OptimisationResults,
    crate_prefix: &str,
) -> TokenStream {
    let crate_prefix = format_ident!("{}", crate_prefix);

    let palettes = results.optimised_palettes.iter().map(|palette| {
        let colours = palette
            .clone()
            .into_iter()
            .map(|colour| colour.to_rgb15())
            .chain(iter::repeat(0))
            .take(16);

        quote! {
            #crate_prefix::display::palette16::Palette16::new([
                #(#colours),*
            ])
        }
    });

    quote! {
        pub const PALETTES: &[#crate_prefix::display::palette16::Palette16] = &[#(#palettes),*];
    }
}

pub(crate) fn generate_code(
    output_variable_name: &str,
    results: &Palette16OptimisationResults,
    image: &Image,
    image_filename: &str,
    tile_size: TileSize,
    crate_prefix: String,
    assignment_offset: Option<usize>,
) -> TokenStream {
    let crate_prefix = format_ident!("{}", crate_prefix);
    let output_variable_name = format_ident!("{}", output_variable_name);

    let (tile_data, assignments) = if let Some(assignment_offset) = assignment_offset {
        let mut tile_data = Vec::new();

        add_image_to_tile_data(&mut tile_data, image, tile_size, results, assignment_offset);

        let tile_data = collapse_to_4bpp(&tile_data);

        let num_tiles = image.width * image.height / tile_size.to_size().pow(2);

        let assignments = results
            .assignments
            .iter()
            .skip(assignment_offset)
            .take(num_tiles)
            .map(|&x| x as u8)
            .collect();

        (tile_data, assignments)
    } else {
        let mut tile_data = Vec::new();

        add_image_256_to_tile_data(&mut tile_data, image, tile_size, results);

        (tile_data, vec![])
    };

    let data = ByteString(&tile_data);

    quote! {
        #[allow(non_upper_case_globals)]
        pub const #output_variable_name: #crate_prefix::display::tile_data::TileData = {
            const _: &[u8] = include_bytes!(#image_filename);

            const TILE_DATA: &[u8] = {
                pub struct AlignedAs<Align, Bytes: ?Sized> {
                    pub _align: [Align; 0],
                    pub bytes: Bytes,
                }

                const ALIGNED: &AlignedAs<u16, [u8]> = &AlignedAs {
                    _align: [],
                    bytes: *#data,
                };

                &ALIGNED.bytes
            };

            const PALETTE_ASSIGNMENT: &[u8] = &[
                #(#assignments),*
            ];

            #crate_prefix::display::tile_data::TileData::new(TILE_DATA, PALETTE_ASSIGNMENT)
        };
    }
}
