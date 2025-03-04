use std::{error::Error, path::Path};

use asefile::AnimationDirection;
use image::{DynamicImage, GenericImageView};
use snafu::{Snafu, ensure};
use syn::{LitStr, Token, parse::Parse};

use crate::{OUT_DIR_TOKEN, aseprite, colour::Colour, get_out_dir, palette16::Palette16};

pub const TRANSPARENT_COLOUR: Colour = Colour::from_rgb(255, 0, 255, 0);

impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let files = input.parse_terminated(<LitStr as Parse>::parse, Token![,])?;
        let files = files
            .iter()
            .map(LitStr::value)
            .map(|x| x.replace(OUT_DIR_TOKEN, &get_out_dir(&x)))
            .collect();

        Ok(Input { files })
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

pub struct Input {
    pub files: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Tag {
    pub name: String,
    pub from: u32,
    pub to: u32,
    pub animation_type: AnimationDirection,
}

pub struct Expanded {
    pub sprites: Vec<DynamicImage>,
    pub tags: Vec<Tag>,
}

#[derive(Clone, Debug)]
pub struct Sprite {
    pub size: (u32, u32),
    pub data: Vec<Colour>,
}

impl Sprite {
    pub fn palette(&self) -> Option<Palette16> {
        let mut palette = Palette16::new();
        for &colour in self.data.iter() {
            if !palette.try_add_colour(if !colour.is_transparent() {
                colour
            } else {
                TRANSPARENT_COLOUR
            }) {
                return None;
            }
        }
        Some(palette)
    }
}

pub struct PreOptimisation {
    pub sprites: Vec<Sprite>,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Snafu)]
#[snafu(display("The sprite size ({x}, {y}) is invalid"))]
struct InvalidSpriteSize {
    x: u32,
    y: u32,
}

impl Input {
    pub fn to_expanded(&self) -> Result<Expanded, Box<dyn Error>> {
        let mut tag_index = 0;
        let mut sprites = Vec::new();
        let mut tags = Vec::new();

        for (image, tag) in self
            .files
            .iter()
            .map(|x| aseprite::generate_from_file(Path::new(x)))
        {
            for tag in tag {
                tags.push(Tag {
                    name: tag.name().to_string(),
                    from: tag.from_frame() + tag_index,
                    to: tag.to_frame() + tag_index,
                    animation_type: tag.animation_direction(),
                });
            }
            tag_index += u32::try_from(image.len())?;
            for image in image {
                let size = image.dimensions();
                ensure!(
                    valid_sprite_size(size.0, size.1),
                    InvalidSpriteSizeSnafu {
                        x: size.0,
                        y: size.1
                    }
                );

                sprites.push(image);
            }
        }

        Ok(Expanded { sprites, tags })
    }
}

impl Expanded {
    pub fn to_pre_optimisation(&self) -> Result<PreOptimisation, Box<dyn Error>> {
        Ok(PreOptimisation {
            tags: self.tags.clone(),
            sprites: self
                .sprites
                .iter()
                .map(|sprite| {
                    let size = sprite.dimensions();

                    Sprite {
                        size,
                        data: sprite
                            .pixels()
                            .map(|(_, _, colour)| {
                                Colour::from_rgb(colour.0[0], colour.0[1], colour.0[2], colour.0[3])
                            })
                            .collect(),
                    }
                })
                .collect(),
        })
    }
}
