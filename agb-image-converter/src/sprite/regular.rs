use std::error::Error;

use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::parse_macro_input;

use crate::{colour::Colour, palette16::Palette16, ByteString, Palette16Optimiser};

use super::common::{Input, PreOptimisation, Tag, TRANSPARENT_COLOUR};

pub fn include_regular(tokens: TokenStream) -> TokenStream {
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
        .to_optimised()?
        .to_output()?;

    Ok(quote! {#output}.into())
}

#[derive(Clone, Debug)]
struct SpriteIndexed {
    size: (u32, u32),
    data: Vec<u8>,
    palette: u32,
}

impl SpriteIndexed {
    fn to_compacted(&self) -> SpriteCompacted {
        let compacted = (0..self.size.1 / 8)
            .flat_map(move |y| (0..self.size.0 / 8).map(move |x| (x, y)))
            .flat_map(|(tile_x, tile_y)| {
                (0..8)
                    .flat_map(move |y| (0..4).map(move |x| (x, y)))
                    .map(move |(x, y)| {
                        let idx = tile_x * 8 + x * 2 + (tile_y * 8 + y) * self.size.0;
                        self.data[idx as usize] | (self.data[idx as usize + 1] << 4)
                    })
            })
            .collect();

        SpriteCompacted {
            size: self.size,
            palette: self.palette,
            data: compacted,
        }
    }
}

struct Optimised {
    palettes: Vec<Palette16>,
    sprites: Vec<SpriteIndexed>,
    tags: Vec<Tag>,
}

struct SpriteCompacted {
    size: (u32, u32),
    data: Vec<u8>,
    palette: u32,
}

struct Output {
    palettes: Vec<Palette16>,
    sprites: Vec<SpriteCompacted>,
    tags: Vec<Tag>,
}

#[derive(snafu::Snafu, Debug)]
struct TooManyColoursInSprite {}

impl PreOptimisation {
    fn to_optimised(&self) -> Result<Optimised, Box<dyn Error>> {
        let mut optimiser = Palette16Optimiser::new(Some(TRANSPARENT_COLOUR));
        for sprite in self.sprites.iter() {
            optimiser.add_palette(sprite.palette().ok_or(TooManyColoursInSprite {})?);
        }

        let optimised_palettes = optimiser.optimise_palettes()?;

        Ok(Optimised {
            sprites: self
                .sprites
                .iter()
                .enumerate()
                .map(|(idx, sprite)| {
                    let palette_idx = optimised_palettes.assignments[idx];
                    let palette = &optimised_palettes.optimised_palettes[palette_idx];

                    SpriteIndexed {
                        size: sprite.size,
                        data: sprite
                            .data
                            .iter()
                            .map(|&colour| {
                                palette.colour_index(if !colour.is_transparent() {
                                    colour
                                } else {
                                    TRANSPARENT_COLOUR
                                })
                            })
                            .collect(),
                        palette: palette_idx as u32,
                    }
                })
                .collect(),
            tags: self.tags.clone(),
            palettes: optimised_palettes.optimised_palettes,
        })
    }
}

impl Optimised {
    fn to_output(&self) -> Result<Output, Box<dyn Error>> {
        Ok(Output {
            palettes: self.palettes.clone(),
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
            let palette_idx = sprite.palette as usize;

            quote! {
                unsafe { Sprite::new(&PALETTES[#palette_idx], align_bytes!(u16, #data), Size::from_width_height(#x, #y)) }
            }
        });

        let palettes = self.palettes.iter().map(|palette| {
            let mut colours: Vec<_> = palette.colours().copied().map(Colour::to_rgb15).collect();
            colours.resize(16, 0);
            quote! {
                Palette16::new([#(#colours),*])
            }
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

        tokens.extend(quote! {
            static PALETTES: &[Palette16] = &[#(#palettes),*];
            static SPRITES: &[Sprite] = &[#(#sprites),*];

            #(#tags)*
        });
    }
}
