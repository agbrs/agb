use std::path::Path;

use asefile::AsepriteFile;
use image::DynamicImage;

use super::Tag;

pub fn generate_from_file(filename: &Path) -> (Vec<DynamicImage>, Vec<Tag>) {
    let ase = AsepriteFile::read_file(filename).expect("Aseprite file should exist");

    let mut images = Vec::new();
    let mut tags = Vec::new();

    for frame in 0..ase.num_frames() {
        let image = ase.frame(frame).image();

        images.push(DynamicImage::ImageRgba8(image))
    }

    for tag in 0..ase.num_tags() {
        let tag = ase.tag(tag);

        tags.push(Tag {
            name: tag.name().to_owned(),
            from_frame: tag.from_frame(),
            to_frame: tag.to_frame(),
            animation_direction: match tag.animation_direction() {
                asefile::AnimationDirection::Forward => super::AnimationDirection::Forward,
                asefile::AnimationDirection::Reverse => super::AnimationDirection::Reverse,
                asefile::AnimationDirection::PingPong => super::AnimationDirection::PingPong,
            },
        });
    }

    (images, tags)
}
