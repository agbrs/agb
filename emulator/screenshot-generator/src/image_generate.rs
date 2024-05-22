use image::{DynamicImage, GenericImage, Rgba};

const WIDTH: usize = 240;
const HEIGHT: usize = 160;

pub fn generate_image(video_buffer: &[u32]) -> DynamicImage {
    let mut dynamic_image = DynamicImage::new(
        WIDTH.try_into().unwrap(),
        HEIGHT.try_into().unwrap(),
        image::ColorType::Rgba8,
    );
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let video_pixel = video_buffer[x + y * WIDTH];
            let mut pixels = video_pixel.to_le_bytes();
            pixels[3] = 255;

            dynamic_image.put_pixel(x.try_into().unwrap(), y.try_into().unwrap(), Rgba(pixels));
        }
    }

    dynamic_image
}
