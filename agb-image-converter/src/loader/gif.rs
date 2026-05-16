use std::{fs::File, path::Path};

use image::DynamicImage;

use image::AnimationDecoder;

use super::Tag;

pub fn generate_from_file(filename: &Path) -> (Vec<DynamicImage>, Vec<Tag>) {
    let file = File::open(filename).expect("Given file should be openable");
    let image = image::codecs::gif::GifDecoder::new(file).expect("Gif should be decodable");
    let tag_name = super::create_tag_name(filename);

    let frames = image
        .into_frames()
        .collect_frames()
        .expect("Should be able to get frames from gif");

    let frames: Vec<_> = frames
        .into_iter()
        .map(|x| DynamicImage::ImageRgba8(x.into_buffer()))
        .collect();

    let tag = Tag {
        name: tag_name.to_owned(),
        from_frame: 0,
        to_frame: frames.len() as u32 - 1,
        animation_direction: super::AnimationDirection::Forward,
    };

    (frames, vec![tag])
}
