use std::ffi::CStr;

use thiserror::Error;

pub static NO_LOGGER: Logger = generate_no_logger();

pub enum LogLevel {
    Fatal,
    Error,
    Warn,
    Info,
    Debug,
    Stub,
    GameError,
    Unknown,
}

#[derive(Debug, Error)]
#[error("A log level of {provided_log_level} does not match any known log level")]
pub struct LogLevelIsNotValid {
    provided_log_level: mgba_sys::mLogLevel,
}

impl TryFrom<mgba_sys::mLogLevel> for LogLevel {
    type Error = LogLevelIsNotValid;

    fn try_from(value: mgba_sys::mLogLevel) -> Result<Self, LogLevelIsNotValid> {
        Ok(match value {
            mgba_sys::mLogLevel_mLOG_FATAL => LogLevel::Fatal,
            mgba_sys::mLogLevel_mLOG_ERROR => LogLevel::Error,
            mgba_sys::mLogLevel_mLOG_WARN => LogLevel::Warn,
            mgba_sys::mLogLevel_mLOG_INFO => LogLevel::Info,
            mgba_sys::mLogLevel_mLOG_DEBUG => LogLevel::Debug,
            mgba_sys::mLogLevel_mLOG_STUB => LogLevel::Stub,
            mgba_sys::mLogLevel_mLOG_GAME_ERROR => LogLevel::GameError,
            _ => {
                return Err(LogLevelIsNotValid {
                    provided_log_level: value,
                })
            }
        })
    }
}

const fn generate_no_logger() -> Logger {
    Logger {
        logger: mgba_sys::mLogger {
            log: Some(no_log),
            filter: std::ptr::null_mut(),
        },
        log: None,
    }
}

#[repr(C)]
pub struct Logger {
    logger: mgba_sys::mLogger,
    log: Option<fn(&str, LogLevel, String)>,
}

impl Logger {
    pub(crate) fn to_mgba(&'static self) -> *mut mgba_sys::mLogger {
        (self as *const Logger)
            .cast::<mgba_sys::mLogger>()
            .cast_mut()
    }

    pub const fn new(logger: fn(&str, LogLevel, String)) -> Self {
        Logger {
            logger: mgba_sys::mLogger {
                log: Some(log_string_wrapper),
                filter: std::ptr::null_mut(),
            },
            log: Some(logger),
        }
    }
}

extern "C" fn log_string_wrapper(
    logger: *mut mgba_sys::mLogger,
    category: i32,
    level: u32,
    format: *const i8,
    args: VaArgs,
) {
    let logger = logger.cast::<Logger>();
    if let Some(logger) = unsafe { &(*logger).log } {
        let s = convert_to_string(format, args);

        if let Some(s) = s {
            let category_c_name = unsafe { mgba_sys::mLogCategoryName(category) };
            const UNKNOWN: &str = "Unknown";
            let category_name = if category_c_name.is_null() {
                UNKNOWN
            } else {
                unsafe { CStr::from_ptr(category_c_name).to_str() }.unwrap_or(UNKNOWN)
            };

            logger(
                category_name,
                LogLevel::try_from(level).unwrap_or(LogLevel::Unknown),
                s,
            );
        }
    }
}

#[cfg(unix)]
type VaArgs = *mut mgba_sys::__va_list_tag;

#[cfg(windows)]
type VaArgs = mgba_sys::va_list;

extern "C" {
    fn vsnprintf(
        s: *mut libc::c_char,
        n: libc::size_t,
        format: *const libc::c_char,
        va_args: VaArgs,
    ) -> std::ffi::c_int;
}

fn convert_to_string(format: *const i8, var_args: VaArgs) -> Option<String> {
    const BUFFER_SIZE: usize = 1024;

    let mut string = vec![0u8; BUFFER_SIZE];

    let count = unsafe { vsnprintf(string.as_mut_ptr().cast(), BUFFER_SIZE, format, var_args) };
    if count < 0 {
        return None;
    }
    // The last byte is always null, so guarentee we can remove that. If we
    // wrote too much in this sprint then we can at least partially recover the
    // string.
    string.truncate(string.len() - 1);
    string.truncate(count as usize);

    String::from_utf8(string).ok()
}

extern "C" fn no_log(
    _l: *mut mgba_sys::mLogger,
    _category: i32,
    _level: u32,
    _format: *const i8,
    _var_args: VaArgs,
) {
}
