mod log;
mod vfile;

use std::{ptr::NonNull, sync::atomic::{AtomicBool, Ordering}};

pub use log::{Logger, LogLevel};
pub use vfile::{file::FileBacked, memory::MemoryBacked, shared::Shared, MapFlag, VFile};

use vfile::VFileAlloc;

pub struct MCore {
    core: NonNull<mgba_sys::mCore>,
    video_buffer: Box<[u32]>,
}

impl Drop for MCore {
    fn drop(&mut self) {
        unsafe { self.core.as_ref().deinit.unwrap()(self.core.as_ptr()) }
    }
}

const SAMPLE_RATE: f64 = 44100.0;

macro_rules! call_on_core {
    ($core:expr => $fn_name:ident($($arg:expr),* $(,)?)) => {
        $core.as_ref().$fn_name.unwrap()($core.as_ptr(), $($arg),*)
    };
}

static GLOBAL_LOGGER_HAS_BEEN_INITIALISED: AtomicBool = AtomicBool::new(false);

pub fn set_global_default_logger(logger: &'static Logger) {
    GLOBAL_LOGGER_HAS_BEEN_INITIALISED.store(true, Ordering::SeqCst);
    unsafe { mgba_sys::mLogSetDefaultLogger(logger.to_mgba()) }
}

impl MCore {
    pub fn new() -> Option<Self> {
        if !GLOBAL_LOGGER_HAS_BEEN_INITIALISED.load(Ordering::SeqCst) {
            set_global_default_logger(&log::NO_LOGGER);
        }

        let core = unsafe { mgba_sys::GBACoreCreate() };
        let core = NonNull::new(core)?;

        unsafe { mgba_sys::mCoreInitConfig(core.as_ptr(), std::ptr::null()) };

        unsafe { call_on_core!(core=>init()) };

        let (mut width, mut height) = (0, 0);

        unsafe { call_on_core!(core=>desiredVideoDimensions(&mut width, &mut height)) };

        let mut video_buffer = vec![
            0;
            (width * height * mgba_sys::BYTES_PER_PIXEL) as usize
                / std::mem::size_of::<u32>()
        ]
        .into_boxed_slice();

        unsafe {
            call_on_core!(
                core=>setVideoBuffer(
                    video_buffer.as_mut_ptr(),
                    width as usize
                )
            )
        }

        unsafe { call_on_core!(core=>reset()) };

        unsafe { call_on_core!(core=>setAudioBufferSize(0x4000)) };

        unsafe {
            mgba_sys::blip_set_rates(
                call_on_core!(core=>getAudioChannel(0)),
                call_on_core!(core=>frequency()) as f64,
                SAMPLE_RATE,
            )
        }
        unsafe {
            mgba_sys::blip_set_rates(
                call_on_core!(core=>getAudioChannel(1)),
                call_on_core!(core=>frequency()) as f64,
                SAMPLE_RATE,
            )
        }

        let core_options = mgba_sys::mCoreOptions {
            volume: 0x100,
            useBios: true,
            ..Default::default()
        };

        unsafe { mgba_sys::mCoreConfigLoadDefaults(&mut (*core.as_ptr()).config, &core_options) };
        unsafe { mgba_sys::mCoreLoadConfig(core.as_ptr()) };

        Some(MCore { core, video_buffer })
    }

    pub fn load_rom<V: VFile>(&mut self, vfile: V) {
        let vfile = VFileAlloc::new(vfile);
        unsafe { call_on_core!(self.core=>loadROM(vfile.into_mgba())) };
    }

    pub fn frame(&mut self) {
        unsafe { call_on_core!(self.core=>runFrame()) };
    }

    pub fn step(&mut self) {
        unsafe { call_on_core!(self.core=>step()) };
    }

    pub fn set_keys(&mut self, buttons: u32) {
        unsafe { call_on_core!(self.core=>setKeys(buttons)) };
    }

    pub fn load_save<V: VFile>(&mut self, save_file: V) {
        let save_file = VFileAlloc::new(save_file);
        unsafe {
            call_on_core!(self.core=>loadSave(save_file.into_mgba()));
        }
    }

    pub fn video_buffer(&mut self) -> &[u32] {
        self.video_buffer.as_ref()
    }

    pub fn current_cycle(&mut self) -> u64 {
        unsafe { mgba_sys::mTimingGlobalTime(self.core.as_ref().timing) }
    }

    pub fn read_audio(&mut self, target: &mut [i16]) -> usize {
        let audio_channel_left = unsafe { call_on_core!(self.core=>getAudioChannel(0)) };
        let audio_channel_right = unsafe { call_on_core!(self.core=>getAudioChannel(1)) };

        let samples_available = unsafe { mgba_sys::blip_samples_avail(audio_channel_left) };

        if samples_available > 0 {
            let samples_to_read = samples_available.min(target.len() as i32 / 2);
            let produced = unsafe {
                mgba_sys::blip_read_samples(
                    audio_channel_left,
                    target.as_mut_ptr().cast(),
                    samples_to_read,
                    1,
                )
            };
            unsafe {
                mgba_sys::blip_read_samples(
                    audio_channel_right,
                    target.as_mut_ptr().add(1).cast(),
                    samples_to_read,
                    1,
                )
            };

            return produced.try_into().unwrap();
        }

        0
    }
}

#[cfg(test)]
mod tests {

    static TEST_ROM: &[u8] = include_bytes!("../test.gba");
    static SAVE_TEST_ROM: &[u8] = include_bytes!("../save.gba");

    use super::*;

    #[test]
    fn check_running_game_for_some_frames() {
        let file = MemoryBacked::new_from_slice(TEST_ROM);
        let mut core = MCore::new().unwrap();
        core.load_rom(file);

        for _ in 0..100 {
            core.frame();
        }
    }

    #[test]
    fn check_save_file_is_initialised() {
        let shared_save_file = Shared::new(MemoryBacked::new(Vec::new()));

        {
            let save_file = shared_save_file.clone();
            let rom_file = MemoryBacked::new_from_slice(SAVE_TEST_ROM);

            let mut core = MCore::new().unwrap();
            core.load_rom(rom_file);
            core.load_save(save_file);
            for _ in 0..10 {
                core.frame();
            }
        }

        let save_file = shared_save_file
            .try_into_inner()
            .unwrap_or_else(|_| panic!("the shared references were not released"))
            .into_inner()
            .into_owned();

        assert_eq!(save_file.len(), 32 * 1024, "the save file should be 32 kb");

        assert_eq!(
            save_file[0..128],
            (0..128).collect::<Vec<u8>>(),
            "First 128 bytes should be ascending numbers"
        );
        assert_eq!(
            save_file[128..],
            std::iter::repeat(0xFF)
                .take(save_file.len() - 128)
                .collect::<Vec<u8>>(),
            "Remanider of save should be 0xFF, all ones"
        );
    }
}
