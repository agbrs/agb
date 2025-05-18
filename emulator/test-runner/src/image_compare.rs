use std::path::Path;

use image::{Rgba, io::Reader};

pub struct ComparisonResult {
    matches: bool,
}

impl ComparisonResult {
    pub fn success(&self) -> bool {
        self.matches
    }
}

pub const WIDTH: usize = 240;
pub const HEIGHT: usize = 160;

#[derive(Eq, PartialEq, Clone, Copy)]
struct Rgb15(u16);

impl Rgb15 {
    fn from_rgba(rgba: Rgba<u8>) -> Self {
        let (r, g, b) = (rgba.0[0] as u16, rgba.0[1] as u16, rgba.0[2] as u16);
        Rgb15(((r >> 3) & 31) | (((g >> 3) & 31) << 5) | (((b >> 3) & 31) << 10))
    }
}

pub fn compare_image(
    image: impl AsRef<Path>,
    video_buffer: &[u32],
) -> anyhow::Result<ComparisonResult> {
    let expected = Reader::open(image)?.decode()?;
    let expected_buffer = expected.to_rgba8();

    let (exp_dim_x, exp_dim_y) = expected_buffer.dimensions();
    if exp_dim_x != WIDTH as u32 || exp_dim_y != HEIGHT as u32 {
        return Ok(ComparisonResult { matches: false });
    }

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let video_pixel = video_buffer[x + y * WIDTH];
            let video_pixel = Rgba::from(video_pixel.to_le_bytes());
            let image_pixel = *expected_buffer.get_pixel(x as u32, y as u32);

            if Rgb15::from_rgba(video_pixel) != Rgb15::from_rgba(image_pixel) {
                return Ok(ComparisonResult { matches: false });
            }
        }
    }

    Ok(ComparisonResult { matches: true })
}
