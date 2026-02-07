use std::error::Error;

use asefile::AnimationDirection;
use image::{DynamicImage, GenericImageView};
use snafu::{Snafu, ensure};
use syn::{LitInt, LitStr, Token, parse::Parse};

use crate::{OUT_DIR_TOKEN, aseprite, colour::Colour, get_out_dir, palette16::Palette16};

pub const TRANSPARENT_COLOUR: Colour = Colour::from_rgb(255, 0, 255, 0);

pub struct FileEntry {
    pub path: String,
    pub size_override: Option<(u32, u32)>,
}

impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut files = Vec::new();

        while !input.is_empty() {
            let size_override = if input.peek(LitInt) {
                let lit: LitInt = input.parse()?;
                let digits = lit.base10_digits();
                let suffix = lit.suffix();

                if !suffix.starts_with('x') {
                    return Err(syn::Error::new(
                        lit.span(),
                        format!(
                            "expected size in WIDTHxHEIGHT format, e.g. 32x16, got {digits}{suffix}"
                        ),
                    ));
                }

                let width: u32 = digits
                    .parse()
                    .map_err(|_| syn::Error::new(lit.span(), "invalid width in size override"))?;
                let height: u32 = suffix[1..]
                    .parse()
                    .map_err(|_| syn::Error::new(lit.span(), "invalid height in size override"))?;

                Some((width, height))
            } else {
                None
            };

            let path_lit: LitStr = input.parse()?;
            let path = path_lit
                .value()
                .replace(OUT_DIR_TOKEN, &get_out_dir(&path_lit.value()));

            files.push(FileEntry {
                path,
                size_override,
            });

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

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
    pub files: Vec<FileEntry>,
}

#[derive(Clone, Debug)]
pub struct Tag {
    pub name: String,
    pub from: u32,
    pub to: u32,
    pub animation_type: AnimationDirection,
}

pub struct Expanded {
    pub input_files: Vec<String>,
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
    pub input_files: Vec<String>,
    pub sprites: Vec<Sprite>,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Snafu)]
#[snafu(display("The sprite size ({x}, {y}) is invalid"))]
struct InvalidSpriteSize {
    x: u32,
    y: u32,
}

#[derive(Debug, Snafu)]
#[snafu(display(
    "Frame size ({frame_w}x{frame_h}) is not evenly divisible by target size ({target_w}x{target_h})"
))]
struct FrameNotDivisible {
    frame_w: u32,
    frame_h: u32,
    target_w: u32,
    target_h: u32,
}

fn split_frame(frame: &DynamicImage, target_w: u32, target_h: u32) -> Vec<DynamicImage> {
    let (frame_w, frame_h) = frame.dimensions();
    let cols = frame_w / target_w;
    let rows = frame_h / target_h;

    let mut sub_frames = Vec::with_capacity((cols * rows) as usize);

    for row in 0..rows {
        for col in 0..cols {
            sub_frames.push(frame.crop_imm(col * target_w, row * target_h, target_w, target_h));
        }
    }

    sub_frames
}

impl Input {
    pub fn to_expanded(&self) -> Result<Expanded, Box<dyn Error>> {
        let mut tag_index = 0;
        let mut sprites = Vec::new();
        let mut tags = Vec::new();

        // Resolve paths for both local crate and workspace contexts
        let resolved_files: Vec<_> = self
            .files
            .iter()
            .map(|entry| (crate::resolve_path(&entry.path), entry.size_override))
            .collect();

        for (resolved_path, size_override) in &resolved_files {
            let (images, file_tags) = aseprite::generate_from_file(resolved_path);

            let split_factor = if let Some((target_w, target_h)) = size_override {
                let target_w = *target_w;
                let target_h = *target_h;

                ensure!(
                    valid_sprite_size(target_w, target_h),
                    InvalidSpriteSizeSnafu {
                        x: target_w,
                        y: target_h
                    }
                );

                if let Some(first) = images.first() {
                    let (frame_w, frame_h) = first.dimensions();
                    ensure!(
                        frame_w % target_w == 0 && frame_h % target_h == 0,
                        FrameNotDivisibleSnafu {
                            frame_w,
                            frame_h,
                            target_w,
                            target_h
                        }
                    );
                    (frame_w / target_w) * (frame_h / target_h)
                } else {
                    1
                }
            } else {
                1
            };

            for tag in &file_tags {
                tags.push(Tag {
                    name: tag.name().to_string(),
                    from: tag.from_frame() * split_factor + tag_index,
                    to: (tag.to_frame() + 1) * split_factor - 1 + tag_index,
                    animation_type: tag.animation_direction(),
                });
            }

            let num_original_frames = u32::try_from(images.len())?;
            tag_index += num_original_frames * split_factor;

            for image in &images {
                if split_factor > 1 {
                    let (target_w, target_h) = size_override.unwrap();
                    for sub_frame in split_frame(image, target_w, target_h) {
                        sprites.push(sub_frame);
                    }
                } else {
                    let size = image.dimensions();
                    ensure!(
                        valid_sprite_size(size.0, size.1),
                        InvalidSpriteSizeSnafu {
                            x: size.0,
                            y: size.1
                        }
                    );
                    sprites.push(image.clone());
                }
            }
        }

        Ok(Expanded {
            input_files: resolved_files
                .iter()
                .map(|(path, _)| path.to_string_lossy().into_owned())
                .collect(),
            sprites,
            tags,
        })
    }
}

impl Expanded {
    pub fn to_pre_optimisation(&self) -> Result<PreOptimisation, Box<dyn Error>> {
        Ok(PreOptimisation {
            input_files: self.input_files.clone(),
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
