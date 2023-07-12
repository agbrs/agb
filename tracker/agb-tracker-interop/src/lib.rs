#![cfg_attr(not(feature = "std"), no_std)]

use agb_fixnum::Num;

#[derive(Debug)]
pub struct Track<'a> {
    pub samples: &'a [Sample<'a>],
    pub pattern_data: &'a [PatternSlot],
    pub patterns: &'a [Pattern],
    pub patterns_to_play: &'a [usize],

    pub num_channels: usize,
    pub frames_per_step: u16,
}

#[derive(Debug)]
pub struct Sample<'a> {
    pub data: &'a [u8],
    pub should_loop: bool,
}

#[derive(Debug)]
pub struct Pattern {
    pub length: usize,
    pub start_position: usize,
}

#[derive(Debug)]
pub struct PatternSlot {
    pub volume: Num<i16, 4>,
    pub speed: Num<u32, 8>,
    pub panning: Num<i16, 4>,
    pub sample: usize,
}

pub const SKIP_SLOT: usize = 277;
pub const STOP_CHANNEL: usize = 278;

#[cfg(feature = "quote")]
impl<'a> quote::ToTokens for Track<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use quote::{quote, TokenStreamExt};

        let Track {
            samples,
            pattern_data,
            patterns,
            frames_per_step,
            num_channels,
            patterns_to_play,
        } = self;

        tokens.append_all(quote! {
            {
                use agb_tracker_interop::*;

                const SAMPLES: &[Sample<'static>] = &[#(#samples),*];
                const PATTERN_DATA: &[PatternSlot] = &[#(#pattern_data),*];
                const PATTERNS: &[Pattern] = &[#(#patterns),*];
                const PATTERNS_TO_PLAY: &[usize] = &[#(#patterns_to_play),*];

                Track {
                    samples: SAMPLES,
                    pattern_data: PATTERN_DATA,
                    patterns: PATTERNS,
                    patterns_to_play: PATTERNS_TO_PLAY,

                    frames_per_step: #frames_per_step,
                    num_channels: #num_channels,
                }
            }
        })
    }
}

#[cfg(feature = "quote")]
struct ByteString<'a>(&'a [u8]);
#[cfg(feature = "quote")]
impl quote::ToTokens for ByteString<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use quote::TokenStreamExt;

        tokens.append(proc_macro2::Literal::byte_string(self.0));
    }
}

#[cfg(feature = "quote")]
impl<'a> quote::ToTokens for Sample<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use quote::{quote, TokenStreamExt};

        let self_as_u8s: Vec<_> = self.data.iter().map(|i| *i as u8).collect();
        let samples = ByteString(&self_as_u8s);
        let should_loop = self.should_loop;

        tokens.append_all(quote! {
            {
                use agb_tracker_interop::*;

                #[repr(align(4))]
                struct AlignmentWrapper<const N: usize>([u8; N]);

                const SAMPLE_DATA: &[u8] = &AlignmentWrapper(*#samples).0;
                agb_tracker_interop::Sample { data: SAMPLE_DATA, should_loop: #should_loop }
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

        let Pattern {
            length,
            start_position,
        } = self;

        tokens.append_all(quote! {
            {
                use agb_tracker_interop::*;

                Pattern {
                    length: #length,
                    start_position: #start_position,
                }
            }
        })
    }
}
