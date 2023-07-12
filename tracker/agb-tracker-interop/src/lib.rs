#![cfg_attr(not(feature = "std"), no_std)]

use agb_fixnum::Num;

#[derive(Debug)]
pub struct Track<'a> {
    pub samples: &'a [Sample<'a>],
    pub pattern_data: &'a [PatternSlot],
    pub patterns: &'a [Pattern],
}

#[derive(Debug)]
pub struct Sample<'a> {
    pub data: &'a [u8],
}

#[derive(Debug)]
pub struct Pattern {
    pub num_channels: usize,
}

#[derive(Debug)]
pub struct PatternSlot {
    pub volume: Num<i16, 4>,
    pub speed: Num<u32, 8>,
    pub panning: Num<i16, 4>,
    pub sample: usize,
}

#[cfg(feature = "quote")]
impl<'a> quote::ToTokens for Track<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use quote::{quote, TokenStreamExt};

        let samples = self.samples;
        let pattern_data = self.pattern_data;
        let patterns = self.patterns;

        tokens.append_all(quote! {
            {
                use agb_tracker_interop::*;

                const SAMPLES: &[Sample<'static>] = &[#(#samples),*];
                const PATTERN_DATA: &[PatternSlot] = &[#(#pattern_data),*];
                const PATTERNS: &[Pattern] = &[#(#patterns),*];

                Track {
                    samples: SAMPLES,
                    pattern_data: PATTERN_DATA,
                    patterns: PATTERNS,
                }
            }
        })
    }
}

#[cfg(feature = "quote")]
impl<'a> quote::ToTokens for Sample<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use quote::{quote, TokenStreamExt};

        let self_as_u8s = self.data.iter().map(|i| *i as u8);

        tokens.append_all(quote! {
            {
                use agb_tracker_interop::*;

                const SAMPLE_DATA: &[u8] = &[#(#self_as_u8s),*];
                agb_tracker_interop::Sample { data: SAMPLE_DATA }
            }
        });
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for PatternSlot {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use quote::{quote, TokenStreamExt};

        let PatternSlot {
            volume,
            speed,
            panning,
            sample,
        } = &self;

        let volume = volume.to_raw();
        let speed = speed.to_raw();
        let panning = panning.to_raw();

        tokens.append_all(quote! {
            {
                use agb_tracker::__private::*;
                use agb::fixnum::Num;

                PatternSlot {
                    volume: Num::from_raw(#volume),
                    speed: Num::from_raw(#speed),
                    panning: Num::from_raw(#panning),
                    sample: #sample,
                }
            }
        });
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for Pattern {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use quote::{quote, TokenStreamExt};

        let num_channels = self.num_channels;

        tokens.append_all(quote! {
            {
                use agb_tracker_interop::*;

                Pattern {
                    num_channels: #num_channels,
                }
            }
        })
    }
}
