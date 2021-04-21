use std::path;

use image::GenericImageView;

use crate::colour::Colour;

pub(crate) struct Image {
    pub width: usize,
    pub height: usize,
    colour_data: Vec<Colour>,
}

impl Image {
    pub fn load_from_file(image_path: &path::Path) -> Self {
        let img = image::open(image_path).expect("Expected image to exist");
        let (width, height) = img.dimensions();

        let width = width as usize;
        let height = height as usize;

        let mut colour_data = Vec::with_capacity(width * height);

        for (_, _, pixel) in img.pixels() {
            colour_data.push(Colour::from_rgb(pixel[0], pixel[1], pixel[2]));
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
