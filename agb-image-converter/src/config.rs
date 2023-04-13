use std::collections::HashMap;

use crate::{Colour, Colours};

pub(crate) trait Config {
    fn crate_prefix(&self) -> String;
    fn images(&self) -> HashMap<String, &dyn Image>;
    fn transparent_colour(&self) -> Option<Colour>;
}

pub(crate) trait Image {
    fn filename(&self) -> String;
    fn colours(&self) -> Colours;
}
