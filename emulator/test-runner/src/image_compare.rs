use std::path::Path;

use image::{io::Reader, DynamicImage};

pub struct ComparisonResult {
    matches: bool,
    image: DynamicImage,
}

impl ComparisonResult {
    pub fn success(&self) -> bool {
        self.matches
    }

    pub fn image(&self) -> &DynamicImage {
        &self.image
    }
}

const WIDTH: usize = 240;
const HEIGHT: usize = 160;

fn convert_rgba_to_nearest_gba_colour(c: [u8; 4]) -> [u8; 4] {
    let mut n = c;
    n.iter_mut()
        .for_each(|a| *a = ((((*a as u32 >> 3) << 3) * 0x21) >> 5) as u8);
    n
}

pub fn compare_image(
    image: impl AsRef<Path>,
    video_buffer: &[u32],
) -> anyhow::Result<ComparisonResult> {
    let expected = Reader::open(image)?.decode()?;
    let expected_buffer = expected.to_rgba8();

    let (exp_dim_x, exp_dim_y) = expected_buffer.dimensions();
    if exp_dim_x != WIDTH as u32 || exp_dim_y != HEIGHT as u32 {
        return Ok(ComparisonResult {
            matches: false,
            image: expected,
        });
    }

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let video_pixel = video_buffer[x + y * WIDTH];
            let image_pixel = expected_buffer.get_pixel(x as u32, y as u32);
            let image_pixel = convert_rgba_to_nearest_gba_colour(image_pixel.0);

            if image_pixel[0..3] != video_pixel.to_le_bytes()[0..3] {
                return Ok(ComparisonResult {
                    matches: false,
                    image: expected,
                });
            }
        }
    }

    Ok(ComparisonResult {
        matches: true,
        image: expected,
    })
}
