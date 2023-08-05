#![cfg_attr(not(feature = "std"), no_std)]

use agb_fixnum::Num;

#[derive(Debug)]
pub struct Track<'a> {
    pub samples: &'a [Sample<'a>],
    pub envelopes: &'a [Envelope<'a>],
    pub pattern_data: &'a [PatternSlot],
    pub patterns: &'a [Pattern],
    pub patterns_to_play: &'a [usize],

    pub num_channels: usize,
    pub frames_per_tick: Num<u32, 8>,
    pub ticks_per_step: u32,
    pub repeat: usize,
}

#[derive(Debug)]
pub struct Sample<'a> {
    pub data: &'a [u8],
    pub should_loop: bool,
    pub restart_point: u32,
    pub volume: Num<i16, 8>,
    pub volume_envelope: Option<usize>,
}

#[derive(Debug)]
pub struct Pattern {
    pub length: usize,
    pub start_position: usize,
}

#[derive(Debug)]
pub struct PatternSlot {
    pub speed: Num<u16, 8>,
    pub sample: u16,
    pub effect1: PatternEffect,
    pub effect2: PatternEffect,
}

#[derive(Debug)]
pub struct Envelope<'a> {
    pub amount: &'a [Num<i16, 8>],
    pub sustain: Option<usize>,
    pub loop_start: Option<usize>,
    pub loop_end: Option<usize>,
}

#[derive(Debug, Default)]
pub enum PatternEffect {
    /// Don't play an effect
    #[default]
    None,
    /// Stops playing the current note
    Stop,
    /// Plays an arpeggiation of three notes in one row, cycling betwen the current note, current note + first speed, current note + second speed
    Arpeggio(Num<u16, 8>, Num<u16, 8>),
    Panning(Num<i16, 4>),
    Volume(Num<i16, 8>),
    VolumeSlide(Num<i16, 8>),
    FineVolumeSlide(Num<i16, 8>),
    NoteCut(u32),
    Portamento(Num<u16, 12>),
    /// Slide each tick the first amount to at most the second amount
    TonePortamento(Num<u16, 12>, Num<u16, 12>),
    SetTicksPerStep(u32),
    SetFramesPerTick(Num<u32, 8>),
    SetGlobalVolume(Num<i32, 8>),
    GlobalVolumeSlide(Num<i32, 8>),
}

#[cfg(feature = "quote")]
impl<'a> quote::ToTokens for Track<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use quote::{quote, TokenStreamExt};

        let Track {
            samples,
            envelopes,
            pattern_data,
            patterns,
            frames_per_tick,
            num_channels,
            patterns_to_play,
            ticks_per_step,
            repeat,
        } = self;

        let frames_per_tick = frames_per_tick.to_raw();

        tokens.append_all(quote! {
            {
                const SAMPLES: &[agb_tracker::__private::agb_tracker_interop::Sample<'static>] = &[#(#samples),*];
                const PATTERN_DATA: &[agb_tracker::__private::agb_tracker_interop::PatternSlot] = &[#(#pattern_data),*];
                const PATTERNS: &[agb_tracker::__private::agb_tracker_interop::Pattern] = &[#(#patterns),*];
                const PATTERNS_TO_PLAY: &[usize] = &[#(#patterns_to_play),*];
                const ENVELOPES: &[agb_tracker::__private::agb_tracker_interop::Envelope<'static>] = &[#(#envelopes),*];

                agb_tracker::Track {
                    samples: SAMPLES,
                    envelopes: ENVELOPES,
                    pattern_data: PATTERN_DATA,
                    patterns: PATTERNS,
                    patterns_to_play: PATTERNS_TO_PLAY,

                    frames_per_tick: agb_tracker::__private::Num::from_raw(#frames_per_tick),
                    num_channels: #num_channels,
                    ticks_per_step: #ticks_per_step,
                    repeat: #repeat,
                }
            }
        })
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for Envelope<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use quote::{quote, TokenStreamExt};

        let Envelope {
            amount,
            sustain,
            loop_start,
            loop_end,
        } = self;

        let amount = amount.iter().map(|value| {
            let value = value.to_raw();
            quote! { agb_tracker::__private::Num::from_raw(#value) }
        });

        let sustain = match sustain {
            Some(value) => quote!(Some(#value)),
            None => quote!(None),
        };
        let loop_start = match loop_start {
            Some(value) => quote!(Some(#value)),
            None => quote!(None),
        };
        let loop_end = match loop_end {
            Some(value) => quote!(Some(#value)),
            None => quote!(None),
        };

        tokens.append_all(quote! {
            {
                const AMOUNTS: &[agb_tracker::__private::Num<i16, 8>] = &[#(#amount),*];

                agb_tracker::__private::agb_tracker_interop::Envelope {
                    amount: AMOUNTS,
                    sustain: #sustain,
                    loop_start: #loop_start,
                    loop_end: #loop_end,
                }
            }
        });
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

        let Sample {
            data,
            should_loop,
            restart_point,
            volume,
            volume_envelope,
        } = self;

        let volume_envelope = match volume_envelope {
            Some(index) => quote!(Some(#index)),
            None => quote!(None),
        };

        let samples = ByteString(data);
        let volume = volume.to_raw();

        tokens.append_all(quote! {
            {
                #[repr(align(4))]
                struct AlignmentWrapper<const N: usize>([u8; N]);

                const SAMPLE_DATA: &[u8] = &AlignmentWrapper(*#samples).0;
                agb_tracker::__private::agb_tracker_interop::Sample {
                    data: SAMPLE_DATA,
                    should_loop: #should_loop,
                    restart_point: #restart_point,
                    volume: agb_tracker::__private::Num::from_raw(#volume),
                    volume_envelope: #volume_envelope,
                }
            }
        });
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for PatternSlot {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use quote::{quote, TokenStreamExt};

        let PatternSlot {
            speed,
            sample,
            effect1,
            effect2,
        } = &self;

        let speed = speed.to_raw();

        tokens.append_all(quote! {
            agb_tracker::__private::agb_tracker_interop::PatternSlot {
                speed: agb_tracker::__private::Num::from_raw(#speed),
                sample: #sample,
                effect1: #effect1,
                effect2: #effect2,
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
            agb_tracker::__private::agb_tracker_interop::Pattern {
                length: #length,
                start_position: #start_position,
            }
        })
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for PatternEffect {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use quote::{quote, TokenStreamExt};

        let type_bit = match self {
            PatternEffect::None => quote! { None },
            PatternEffect::Stop => quote! { Stop },
            PatternEffect::Arpeggio(first, second) => {
                let first = first.to_raw();
                let second = second.to_raw();
                quote! { Arpeggio(agb_tracker::__private::Num::from_raw(#first), agb_tracker::__private::Num::from_raw(#second)) }
            }
            PatternEffect::Panning(panning) => {
                let panning = panning.to_raw();
                quote! { Panning(agb_tracker::__private::Num::from_raw(#panning))}
            }
            PatternEffect::Volume(volume) => {
                let volume = volume.to_raw();
                quote! { Volume(agb_tracker::__private::Num::from_raw(#volume))}
            }
            PatternEffect::VolumeSlide(amount) => {
                let amount = amount.to_raw();
                quote! { VolumeSlide(agb_tracker::__private::Num::from_raw(#amount))}
            }
            PatternEffect::FineVolumeSlide(amount) => {
                let amount = amount.to_raw();
                quote! { FineVolumeSlide(agb_tracker::__private::Num::from_raw(#amount))}
            }
            PatternEffect::NoteCut(wait) => quote! { NoteCut(#wait) },
            PatternEffect::Portamento(amount) => {
                let amount = amount.to_raw();
                quote! { Portamento(agb_tracker::__private::Num::from_raw(#amount))}
            }
            PatternEffect::TonePortamento(amount, target) => {
                let amount = amount.to_raw();
                let target = target.to_raw();
                quote! { TonePortamento(agb_tracker::__private::Num::from_raw(#amount), agb_tracker::__private::Num::from_raw(#target))}
            }
            PatternEffect::SetTicksPerStep(new_ticks) => {
                quote! { SetTicksPerStep(#new_ticks) }
            }
            PatternEffect::SetFramesPerTick(new_frames_per_tick) => {
                let amount = new_frames_per_tick.to_raw();
                quote! { SetFramesPerTick(agb_tracker::__private::Num::from_raw(#amount)) }
            }
            PatternEffect::SetGlobalVolume(amount) => {
                let amount = amount.to_raw();
                quote! { SetGlobalVolume(agb_tracker::__private::Num::from_raw(#amount)) }
            }
            PatternEffect::GlobalVolumeSlide(amount) => {
                let amount = amount.to_raw();
                quote! { GlobalVolumeSlide(agb_tracker::__private::Num::from_raw(#amount)) }
            }
        };

        tokens.append_all(quote! {
            agb_tracker::__private::agb_tracker_interop::PatternEffect::#type_bit
        });
    }
}
