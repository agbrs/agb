use super::{KerningData, LetterData, generate_font_tokens};
use proc_macro2::TokenStream;

pub fn load_font(font_data: &[u8], pixels_per_em: f32) -> TokenStream {
    let (letters, line_height, ascent) = load_font_letters(font_data, pixels_per_em);
    generate_font_tokens(letters, line_height, ascent)
}

pub(crate) fn load_font_letters(
    font_data: &[u8],
    pixels_per_em: f32,
) -> (Vec<LetterData>, i32, i32) {
    let font = fontdue::Font::from_bytes(
        font_data,
        fontdue::FontSettings {
            scale: pixels_per_em,
            ..Default::default()
        },
    )
    .expect("Invalid font data");

    let line_metrics = font.horizontal_line_metrics(pixels_per_em).unwrap();

    let line_height = line_metrics.new_line_size as i32;
    let ascent = line_metrics.ascent as i32;

    let mut letters: Vec<_> = font
        .chars()
        .iter()
        .map(|(&c, &index)| (c, index, font.rasterize(c, pixels_per_em)))
        .map(|(c, index, (metrics, bitmap))| {
            let width = metrics.width.div_ceil(8) * 8;
            let height = metrics.height;

            let rendered = if bitmap.is_empty() {
                vec![]
            } else {
                bitmap
                    .chunks(metrics.width)
                    .flat_map(|row| {
                        row.chunks(8).map(|chunk| {
                            let mut output = 0u8;
                            for (i, &value) in chunk.iter().enumerate() {
                                if value > 100 {
                                    output |= 1 << i;
                                }
                            }

                            output
                        })
                    })
                    .collect()
            };

            let mut kerning_data: Vec<_> = font
                .chars()
                .iter()
                .filter_map(|(&left_char, &left_index)| {
                    let kerning = font.horizontal_kern_indexed(
                        left_index.into(),
                        index.into(),
                        pixels_per_em,
                    )?;

                    Some(KerningData {
                        previous_character: left_char,
                        amount: kerning,
                    })
                })
                .collect();

            kerning_data.sort_unstable_by_key(|kd| kd.previous_character);

            LetterData {
                character: c,
                width,
                height,
                rendered,
                xmin: metrics.xmin,
                ymin: metrics.ymin,
                advance_width: metrics.advance_width,
                kerning_data,
            }
        })
        .collect();

    letters.sort_unstable_by_key(|letter| letter.character);

    (letters, line_height, ascent)
}
