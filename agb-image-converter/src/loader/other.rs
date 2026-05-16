use std::path::Path;

use image::DynamicImage;

use super::Tag;

pub fn generate_from_file(filename: &Path) -> (Vec<DynamicImage>, Vec<Tag>) {
    let image = image::open(filename).expect("Image should be decodable");
    let tag_name = super::create_tag_name(filename);

    let tag = Tag {
        name: tag_name.to_owned(),
        from_frame: 0,
        to_frame: 0,
        animation_direction: super::AnimationDirection::Forward,
    };

    (vec![image], vec![tag])
}
