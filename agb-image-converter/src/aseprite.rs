use std::path::Path;

use asefile::{AsepriteFile, Tag};
use image::DynamicImage;

pub fn generate_from_file(filename: &Path) -> (Vec<DynamicImage>, Vec<Tag>) {
    let ase = AsepriteFile::read_file(filename).expect("Aseprite file should exist");

    let mut images = Vec::new();
    let mut tags = Vec::new();

    for frame in 0..ase.num_frames() {
        let image = ase.frame(frame).image();

        images.push(DynamicImage::ImageRgba8(image))
    }

    for tag in 0..ase.num_tags() {
        tags.push(ase.tag(tag).clone())
    }

    (images, tags)
}
