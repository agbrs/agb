use std::{fmt, str::FromStr};

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Colour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl fmt::Debug for Colour {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)?;

        if self.a != 0xff {
            write!(f, "{:02x}", self.a)?;
        }

        Ok(())
    }
}

impl Colour {
    pub fn from_rgb(r: u8, g: u8, b: u8, a: u8) -> Self {
        Colour { r, g, b, a }
    }

    pub fn to_rgb15(self) -> u16 {
        let (r, g, b) = (self.r as u16, self.g as u16, self.b as u16);
        ((r >> 3) & 31) | (((g >> 3) & 31) << 5) | (((b >> 3) & 31) << 10)
    }

    pub fn is_transparent(self) -> bool {
        self.a != 255
    }
}

impl FromStr for Colour {
    type Err = String;

    fn from_str(colour: &str) -> Result<Self, Self::Err> {
        if colour.len() != 6 {
            return Err(format!("Expected colour to be 6 characters, got {colour}"));
        }

        let r = u8::from_str_radix(&colour[0..2], 16).unwrap();
        let g = u8::from_str_radix(&colour[2..4], 16).unwrap();
        let b = u8::from_str_radix(&colour[4..6], 16).unwrap();

        Ok(Colour::from_rgb(r, g, b, 255))
    }
}

#[cfg(test)]
impl quickcheck::Arbitrary for Colour {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Self::from_rgb(
            quickcheck::Arbitrary::arbitrary(g),
            quickcheck::Arbitrary::arbitrary(g),
            quickcheck::Arbitrary::arbitrary(g),
            quickcheck::Arbitrary::arbitrary(g),
        )
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            vec![
                Colour::from_rgb(0, 0, 0, 0),
                Colour::from_rgb(self.r, self.g, self.b, 0),
                *self,
            ]
            .into_iter(),
        )
    }
}
