use std::{collections::HashSet, iter::FromIterator};

use crate::{
    colour::Colour,
    image_loader::Image,
    palette16::{Palette16, Palette16OptimisationResults},
};

pub struct Palette256 {
    colours: HashSet<Colour>,
}

impl Palette256 {
    pub fn new() -> Self {
        Self {
            colours: HashSet::new(),
        }
    }

    pub(crate) fn add_image(&mut self, image: &Image) {
        for y in 0..image.height {
            for x in 0..image.width {
                self.colours.insert(image.colour(x, y));
            }
        }

        assert!(
            self.colours.len() <= 256,
            "Must have at most 256 colours in the palette"
        );
    }

    pub(crate) fn extend_results(
        &self,
        palette16: &Palette16OptimisationResults,
    ) -> Palette16OptimisationResults {
        let optimised_palette_colours: Vec<_> = palette16
            .optimised_palettes
            .iter()
            .flat_map(|p| p.colours())
            .cloned()
            .collect();

        let current_colours_set = HashSet::from_iter(optimised_palette_colours.iter().cloned());
        let new_colours: HashSet<_> = self
            .colours
            .symmetric_difference(&current_colours_set)
            .collect();

        assert!(
            new_colours.len() + optimised_palette_colours.len() <= 256,
            "Cannot optimise 16 colour and 256 colour palettes together, produces too many colours"
        );

        let mut new_palettes = palette16.optimised_palettes.clone();
        new_palettes.resize_with(16, Palette16::new);

        for colour in new_colours {
            for palette in new_palettes.iter_mut() {
                if palette.try_add_colour(*colour) {
                    break;
                }
            }
        }

        Palette16OptimisationResults {
            optimised_palettes: new_palettes,
            assignments: palette16.assignments.clone(),
            transparent_colour: palette16.transparent_colour,
        }
    }
}
