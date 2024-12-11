pub trait CharConfigurator {
    fn switch_palette(&mut self, palette_index: u32);
}

pub struct NullCharConfigurator;

impl CharConfigurator for NullCharConfigurator {
    fn switch_palette(&mut self, _palette_index: u32) {}
}
