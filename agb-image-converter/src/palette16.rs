use crate::colour::Colour;
use std::{collections::HashSet, fmt};

const MAX_COLOURS: usize = 256;
const MAX_COLOURS_PER_PALETTE: usize = 16;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) struct Palette16 {
    colours: Vec<Colour>,
}

impl Palette16 {
    pub fn new() -> Self {
        Palette16 {
            colours: Vec::with_capacity(MAX_COLOURS_PER_PALETTE),
        }
    }

    pub fn add_colour(&mut self, colour: Colour) -> bool {
        if self.colours.contains(&colour) {
            return false;
        }

        if self.colours.len() == MAX_COLOURS_PER_PALETTE {
            panic!("Can have at most 16 colours in a single palette");
        }
        self.colours.push(colour);
        true
    }

    pub fn try_add_colour(&mut self, colour: Colour) -> bool {
        if self.colours.contains(&colour) {
            return true;
        }

        if self.colours.len() == MAX_COLOURS_PER_PALETTE {
            return false;
        }

        self.colours.push(colour);
        true
    }

    pub fn colour_index(&self, colour: Colour) -> u8 {
        // A transparent color is always index 0
        if colour.is_transparent() {
            return 0;
        }

        self.colours
            .iter()
            .position(|c| *c == colour)
            .unwrap_or_else(|| {
                panic!(
                    "Can't get a colour index without it existing, looking for {:?}, got {:?}",
                    colour, self.colours
                )
            }) as u8
    }

    pub fn colours(&self) -> impl Iterator<Item = &Colour> {
        self.colours.iter()
    }

    fn union_length(&self, other: &Palette16) -> usize {
        self.colours
            .iter()
            .chain(&other.colours)
            .collect::<HashSet<_>>()
            .len()
    }

    fn is_satisfied_by(&self, other: &Palette16) -> bool {
        self.colours
            .iter()
            .collect::<HashSet<_>>()
            .is_subset(&other.colours.iter().collect::<HashSet<_>>())
    }
}

impl IntoIterator for Palette16 {
    type Item = Colour;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.colours.into_iter()
    }
}

impl<'a, T> From<T> for Palette16
where
    T: IntoIterator<Item = &'a Colour>,
{
    fn from(value: T) -> Self {
        let mut palette = Palette16::new();
        for colour in value.into_iter() {
            palette.add_colour(*colour);
        }

        palette
    }
}

pub(crate) struct Palette16Optimiser {
    palettes: Vec<Palette16>,
    colours: Vec<Colour>,
    transparent_colour: Option<Colour>,
}

#[derive(Debug)]
pub(crate) struct Palette16OptimisationResults {
    pub optimised_palettes: Vec<Palette16>,
    pub assignments: Vec<usize>,
    pub transparent_colour: Option<Colour>,
}

impl Palette16Optimiser {
    pub fn new(transparent_colour: Option<Colour>) -> Self {
        Palette16Optimiser {
            palettes: vec![],
            colours: Vec::new(),
            transparent_colour,
        }
    }

    pub fn add_palette(&mut self, palette: Palette16) {
        self.palettes.push(palette.clone());

        for colour in palette.colours {
            if self.colours.contains(&colour) {
                continue;
            }

            self.colours.push(colour);
        }

        if self.colours.len() > MAX_COLOURS {
            panic!("Cannot have over 256 colours");
        }
    }

    pub fn optimise_palettes(&self) -> Result<Palette16OptimisationResults, DoesNotFitError> {
        let palettes_to_optimise = self
            .palettes
            .iter()
            .cloned()
            .map(|mut palette| {
                // ensure each palette we're creating the covering for has the transparent colour in it
                palette.add_colour(
                    self.transparent_colour
                        .unwrap_or_else(|| Colour::from_rgb(255, 0, 255, 0)),
                );
                palette
            })
            .collect::<HashSet<Palette16>>()
            .into_iter()
            .map(|palette| palette.colours)
            .collect::<Vec<_>>();

        let packed_palettes =
            pagination_packing::overload_and_remove::<_, _, Vec<_>>(&palettes_to_optimise, 16);

        if packed_palettes.len() > 16 {
            return Err(DoesNotFitError(packed_palettes.len()));
        }

        let optimised_palettes = packed_palettes
            .iter()
            .map(|packed_palette| {
                let colours = packed_palette.unique_symbols(&palettes_to_optimise);
                Palette16::from(colours)
            })
            .collect::<Vec<_>>();

        let mut assignments = vec![0; self.palettes.len()];

        for (i, overall_palette) in self.palettes.iter().enumerate() {
            assignments[i] = optimised_palettes
                .iter()
                .position(|palette| overall_palette.is_satisfied_by(palette))
                .unwrap();
        }

        Ok(Palette16OptimisationResults {
            optimised_palettes,
            assignments,
            transparent_colour: self.transparent_colour,
        })
    }
}

pub struct DoesNotFitError(pub usize);

impl fmt::Display for DoesNotFitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Could not fit colours into palette, needed {} bins but can have at most 16",
            self.0
        )
    }
}

impl fmt::Debug for DoesNotFitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

#[cfg(test)]
mod test {
    use quickcheck::{quickcheck, Arbitrary};

    use super::*;

    quickcheck! {
        fn less_than_256_colours_always_fits(palettes: Vec<Palette16>) -> bool {
            let mut optimiser = Palette16Optimiser::new(None);
            for palette in palettes.clone().into_iter().take(16) {
                optimiser.add_palette(palette);
            }

            let Ok(optimisation_results) = optimiser.optimise_palettes() else {
                return false
            };

            for (i, palette) in palettes.into_iter().take(16).enumerate() {
                let optimised_palette = &optimisation_results.optimised_palettes[optimisation_results.assignments[i]];
                if !palette.is_satisfied_by(optimised_palette) {
                    return false;
                }
            }

            true
        }
    }

    impl Arbitrary for Palette16 {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let mut palette = Palette16::new();

            let size: usize = Arbitrary::arbitrary(g);
            // never entirely fill the palette, will give at most 15 colours
            let size = size.rem_euclid(16);

            for _ in 0..size {
                palette.add_colour(Arbitrary::arbitrary(g));
            }

            palette
        }
    }
}
