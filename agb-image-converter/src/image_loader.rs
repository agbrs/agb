use std::{ffi::OsStr, path};

use image::{DynamicImage, GenericImageView};

use crate::colour::Colour;

pub(crate) struct Image {
    pub width: usize,
    pub height: usize,
    colour_data: Vec<Colour>,
}

impl Image {
    pub fn load_from_file(image_path: &path::Path) -> Self {
        let img = if image_path
            .extension()
            .is_some_and(|extension| extension == OsStr::new("aseprite"))
        {
            let ase =
                asefile::AsepriteFile::read_file(image_path).expect("failed to read aseprite file");
            DynamicImage::ImageRgba8(ase.frame(0).image())
        } else {
            image::open(image_path).expect("Expected image to exist")
        };

        Self::load_from_dyn_image(img)
    }

    pub fn load_from_dyn_image(img: image::DynamicImage) -> Self {
        let (width, height) = img.dimensions();

        let width = width as usize;
        let height = height as usize;

        let mut colour_data = Vec::with_capacity(width * height);

        for (_, _, pixel) in img.pixels() {
            colour_data.push(Colour::from_rgb(pixel[0], pixel[1], pixel[2], pixel[3]));
        }

        Image {
            width,
            height,
            colour_data,
        }
    }

    pub fn colour(&self, x: usize, y: usize) -> Colour {
        self.colour_data[x + y * self.width]
    }
}
