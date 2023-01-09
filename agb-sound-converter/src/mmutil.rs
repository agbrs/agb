use std::{collections::HashMap, ffi::CString, fs, iter, path::PathBuf, sync::Mutex};

static mut MMUTIL_MUTEX: Mutex<()> = Mutex::new(());

pub struct MmConverted {
    pub constants: HashMap<String, i32>,
    pub soundbank_data: Vec<u8>,
}

pub fn mm_convert(inputs: &[PathBuf]) -> MmConverted {
    let dir = tempfile::tempdir().expect("Failed to create temporary directory");

    let soundbank_file_path = dir.path().join("soundbank.bin");
    let header_file_path = dir.path().join("soundbank.h");

    let soundbank_file = CString::new(soundbank_file_path.to_str().unwrap()).unwrap();
    let header_file = CString::new(header_file_path.to_str().unwrap()).unwrap();

    let mut args: Vec<_> = iter::once(&PathBuf::from("dummy"))
        .chain(inputs)
        .map(|arg| {
            CString::new(arg.to_str().expect("Need utf8 filename"))
                .expect("filename cannot contain null bytes")
                .into_raw()
        })
        .collect();

    unsafe {
        let _guard = MMUTIL_MUTEX.lock().unwrap();

        crate::mmutil_sys::MSL_Create(
            args.as_mut_ptr(),
            args.len() as i32,
            soundbank_file.into_raw(),
            header_file.into_raw(),
            1,
        );
    }

    let soundbank_data = fs::read(soundbank_file_path).expect("Failed to read soundbank file");
    let header_data = fs::read_to_string(header_file_path).expect("Failed to read header file");

    let constants = header_data
        .split('\n')
        .filter_map(|line| {
            let split_line: Vec<_> = line.split_ascii_whitespace().collect();

            if split_line.len() != 3 {
                return None;
            }

            Some((
                split_line[1].to_owned(),
                split_line[2].parse::<i32>().unwrap(),
            ))
        })
        .collect();

    MmConverted {
        constants,
        soundbank_data,
    }
}
