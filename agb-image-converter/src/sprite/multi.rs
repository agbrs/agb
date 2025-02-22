use std::{
    collections::{HashMap, HashSet},
    error::Error,
};

use crate::{colour::Colour, ByteString};
use proc_macro::TokenStream;
use quote::{format_ident, ToTokens};
use snafu::prelude::*;
use syn::parse_macro_input;

use super::common::{Input, PreOptimisation, Sprite, Tag};
use quote::quote;

pub fn include_multi(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as Input);

    match process_input(&input) {
        Ok(tokens) => tokens,
        Err(err) => panic!("Failed to generate sprites: {}", err),
    }
}

fn process_input(input: &Input) -> Result<TokenStream, Box<dyn Error>> {
    let output = input
        .to_expanded()?
        .to_pre_optimisation()?
        .to_optimised_multi()?
        .to_output()?;

    Ok(quote! {#output}.into())
}

struct SpriteIndexed {
    size: (u32, u32),
    data: Vec<u8>,
}

struct Optimised {
    palettes: Vec<u16>,
    sprites: Vec<SpriteIndexed>,
    tags: Vec<Tag>,
}

fn generate_palette(sprites: &[Sprite]) -> Vec<u16> {
    let colours: HashSet<_> = sprites
        .iter()
        .flat_map(|x| x.data.iter().copied())
        .filter(|&x| !Colour::is_transparent(x))
        .map(|x| x.to_rgb15())
        .collect();

    let mut palette: Vec<_> = colours.into_iter().collect();
    palette.sort();

    palette
}

#[derive(Debug, Snafu)]
#[snafu(
    display("There are more than 256 colours in this collection of sprites which is unrepresentable.
    Consider splitting this import noting that sprites from different multi palette imports may be unusable."
))]
struct TooManyColoursInSprites;

impl PreOptimisation {
    fn to_optimised_multi(&self) -> Result<Optimised, Box<dyn Error>> {
        let palette = generate_palette(&self.sprites);
        if palette.len() >= 256 {
            return Err(TooManyColoursInSprites.into());
        }

        let palette_length = palette.len().div_ceil(16) * 16;
        let index_offset = 256 - palette_length;

        let palette_index_lookup: HashMap<_, _> = palette
            .iter()
            .copied()
            .enumerate()
            .map(|(idx, c)| {
                (
                    c,
                    u8::try_from(idx + index_offset).expect("palette index is valid u8"),
                )
            })
            .collect();

        let mut palette = palette;
        palette.resize(palette_length, 0);
        let palette = palette;

        let sprites_indexed = self
            .sprites
            .iter()
            .map(|x| SpriteIndexed {
                data: x
                    .data
                    .iter()
                    .map(|c| {
                        if c.is_transparent() {
                            0
                        } else {
                            let c = c.to_rgb15();
                            palette_index_lookup[&c]
                        }
                    })
                    .collect(),
                size: x.size,
            })
            .collect();

        Ok(Optimised {
            palettes: palette,
            sprites: sprites_indexed,
            tags: self.tags.clone(),
        })
    }
}

struct SpriteCompacted {
    data: Vec<u8>,
    size: (u32, u32),
}

struct Output {
    palette: Vec<u16>,
    sprites: Vec<SpriteCompacted>,
    tags: Vec<Tag>,
}

impl SpriteIndexed {
    fn to_compacted(&self) -> SpriteCompacted {
        let compacted = (0..self.size.1 / 8)
            .flat_map(move |y| (0..self.size.0 / 8).map(move |x| (x, y)))
            .flat_map(|(tile_x, tile_y)| {
                (0..8)
                    .flat_map(move |y| (0..8).map(move |x| (x, y)))
                    .map(move |(x, y)| {
                        let idx = tile_x * 8 + x + (tile_y * 8 + y) * self.size.0;
                        self.data[idx as usize]
                    })
            })
            .collect();

        SpriteCompacted {
            size: self.size,
            data: compacted,
        }
    }
}

impl Optimised {
    fn to_output(&self) -> Result<Output, Box<dyn Error>> {
        Ok(Output {
            palette: self.palettes.clone(),
            sprites: self
                .sprites
                .iter()
                .map(SpriteIndexed::to_compacted)
                .collect(),
            tags: self.tags.clone(),
        })
    }
}

impl ToTokens for Output {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let sprites = self.sprites.iter().map(|sprite| {
            let data = ByteString(&sprite.data);
            let x = sprite.size.0 as usize;
            let y = sprite.size.1 as usize;

            quote! {
                unsafe { Sprite::new_multi(&PALETTE, align_bytes!(u16, #data), Size::from_width_height(#x, #y)) }
            }
        });

        let palettes = self.palette.chunks(16).map(|palette| {
            quote! { Palette16::new([#(#palette),*])}
        });

        let tags = self.tags.iter().map(|tag| {
            let ident = format_ident!(
                "{}",
                tag.name
                    .to_ascii_uppercase()
                    .replace(" ", "_")
                    .replace("-", "_")
            );
            let from = tag.from as usize;
            let to = tag.to as usize;
            let len = to - from + 1;
            let direction = tag.animation_type as usize;

            quote! {
                pub static #ident: Tag = Tag::new(unsafe { core::slice::from_raw_parts(SPRITES.as_ptr().add(#from), #len) }, #direction);
            }
        });

        let start = (16 - self.palette.len() / 16) as u32;

        tokens.extend(quote! {
            static PALETTE: PaletteMulti = PaletteMulti::new(#start, &[#(#palettes),*] );
            static SPRITES: &[Sprite] = &[#(#sprites),*];

            #(#tags)*
        });
    }
}
