use agb::{
    display::{GraphicsFrame, Rgb15, tiled::VRAM_MANAGER},
    dma::HBlankDma,
    include_colours,
};

static SKY_GRADIENT: [Rgb15; 160] = include_colours!("gfx/sky-background-gradient.aseprite");

pub fn apply(frame: &mut GraphicsFrame<'_>) {
    HBlankDma::new(
        VRAM_MANAGER.background_palette_colour_dma(0, 0),
        &SKY_GRADIENT,
    )
    .show(frame);
}
