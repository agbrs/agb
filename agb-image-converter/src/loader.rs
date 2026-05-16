use std::path::Path;

use image::DynamicImage;

mod aseprite;
mod gif;

#[derive(Debug, Clone)]
pub struct Tag {
    pub name: String,
    pub from_frame: u32,
    pub to_frame: u32,
    pub animation_direction: AnimationDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationDirection {
    Forward,
    Reverse,
    PingPong,
}

pub fn generate_from_file(filename: &Path) -> (Vec<DynamicImage>, Vec<Tag>) {
    let extension = filename.extension().and_then(|x| x.to_str());

    match extension {
        Some("gif") => gif::generate_from_file(filename),
        _ => aseprite::generate_from_file(filename),
    }
}

fn create_tag_name(filename: &Path) -> &str {
    filename
        .file_stem()
        .and_then(|x| x.to_str())
        .expect("Given filename should have name representable as a string")
}
