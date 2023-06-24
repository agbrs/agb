use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use super::{
    bitmap3::Bitmap3,
    bitmap4::Bitmap4,
    tiled::{Tiled0, Tiled1, Tiled2, VRamManager},
};

pub struct Video();

impl Video {
    pub fn get<Mode: DisplayImplementation>(&mut self) -> Display<'_, Mode> {
        Display(unsafe { Mode::new() }, PhantomData)
    }
}

pub trait DisplayImplementation {
    /// # Safety
    /// * This is safe when only one of these exist at a time.
    /// * This includes all implementations of this trait.
    unsafe fn new() -> Self;
}

pub struct Display<'display, Mode: DisplayImplementation>(Mode, PhantomData<&'display mut Mode>);

impl<M: DisplayImplementation> Deref for Display<'_, M> {
    type Target = M;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<M: DisplayImplementation> DerefMut for Display<'_, M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a, M: DisplayImplementation> Display<'a, M> {
    pub fn change<NextMode: DisplayImplementation>(self) -> Display<'a, NextMode> {
        drop(self);
        Display(unsafe { NextMode::new() }, PhantomData)
    }
}

pub type Tiled0Vram = (Tiled0, VRamManager);

impl DisplayImplementation for Tiled0Vram {
    unsafe fn new() -> Self {
        (unsafe { Tiled0::new() }, VRamManager::new())
    }
}

pub type Tiled1Vram = (Tiled1, VRamManager);

impl DisplayImplementation for Tiled1Vram {
    unsafe fn new() -> Self {
        (unsafe { Tiled1::new() }, VRamManager::new())
    }
}

pub type Tiled2Vram = (Tiled2, VRamManager);

impl DisplayImplementation for Tiled2Vram {
    unsafe fn new() -> Self {
        (unsafe { Tiled2::new() }, VRamManager::new())
    }
}
