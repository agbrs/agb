use core::arch::global_asm;

global_asm!(include_str!("macros.inc"));
global_asm!(include_str!("memcpy.s"));
global_asm!(include_str!("memset.s"));

extern "C" {
    fn __aeabi_memcpy4(dest: *mut u32, src: *const u32, n: usize);
}

pub(crate) unsafe fn memcpy(dest: *mut u32, src: *const u32, n: usize) {
    __aeabi_memcpy4(dest, src, n);
}

#[cfg(test)]
mod test {
    mod memset {
        use core::slice;

        use crate::Gba;
        use alloc::vec;

        extern "C" {
            fn __agbabi_memset(dest: *mut u8, n: usize, v: u8);
            fn __aeabi_memset4(dest: *mut u32, n: usize, v: u8);
        }

        #[test_case]
        fn test_memset_with_different_sizes_and_offsets(_gba: &mut Gba) {
            let mut values = vec![0u8; 100];

            let stored_value = 0x12;

            for n in 0..80 {
                for o in 0..10 {
                    values.fill(0xFF);

                    unsafe {
                        __agbabi_memset(values.as_mut_ptr().add(o), n, stored_value);
                    }

                    for (i, &v) in values.iter().enumerate().take(o) {
                        assert_eq!(v, 0xFF, "underrun at {i}, offset {o}, size {n}");
                    }

                    for (i, &v) in values.iter().enumerate().skip(o).take(n) {
                        assert_eq!(
                            v, stored_value,
                            "incorrect value at {i}, offset {o}, size {n}"
                        );
                    }

                    for (i, &v) in values.iter().enumerate().skip(o + n) {
                        assert_eq!(v, 0xFF, "overrun at {i}, offset {o}, size {n}");
                    }
                }
            }
        }

        #[test_case]
        fn test_memset4_with_different_sizes_and_offsets(_gba: &mut Gba) {
            let mut values = vec![0u32; 100];

            let stored_value = 0x12;

            for n in 0..80 {
                for o in 0..10 {
                    values.fill(0xFF);

                    unsafe {
                        __aeabi_memset4(values.as_mut_ptr().add(o), n * 4, stored_value);
                    }

                    for (i, &v) in values.iter().enumerate().take(o) {
                        assert_eq!(v, 0xFF, "underrun at {i}, offset {o}, size {n}");
                    }

                    for (i, &v) in values.iter().enumerate().skip(o).take(n) {
                        assert_eq!(
                            v, 0x12121212,
                            "incorrect value at {i}, offset {o}, size {n}"
                        );
                    }

                    for (i, &v) in values.iter().enumerate().skip(o + n) {
                        assert_eq!(v, 0xFF, "overrun at {i}, offset {o}, size {n}");
                    }
                }
            }
        }

        #[test_case]
        fn test_memset4_with_non_word_size_sizes_and_offsets(_gba: &mut Gba) {
            let mut values = vec![0u32; 100];

            let stored_value = 0x12;

            for n in 0..80 {
                for o in 0..10 {
                    values.fill(0xFFFFFFFF);

                    unsafe {
                        __aeabi_memset4(values.as_mut_ptr().add(o), n, stored_value);
                    }

                    let values_bytes: &[u8] =
                        unsafe { slice::from_raw_parts(values.as_ptr().cast(), values.len() * 4) };

                    for (i, &v) in values_bytes.iter().enumerate().take(o * 4) {
                        assert_eq!(v, 0xFF, "underrun at {i}, offset {o}, size {n}");
                    }

                    for (i, &v) in values_bytes.iter().enumerate().skip(o * 4).take(n) {
                        assert_eq!(v, 0x12, "incorrect value at {i}, offset {o}, size {n}");
                    }

                    for (i, &v) in values_bytes.iter().enumerate().skip(o * 4 + n) {
                        assert_eq!(v, 0xFF, "overrun at {i}, offset {o}, size {n}");
                    }
                }
            }
        }
    }

    mod memcpy {
        use core::slice;

        use alloc::vec;

        use crate::Gba;

        extern "C" {
            fn __agbabi_memcpy(dest: *mut u8, src: *const u8, n: usize);
            fn __aeabi_memcpy4(dest: *mut u32, src: *const u32, n: usize);
        }

        #[test_case]
        fn test_memcpy4_with_different_sizes(_gba: &mut Gba) {
            let mut input = vec![0u32; 70];
            let mut output = vec![0u32; 70];

            for size in 0..68 {
                for (i, value) in input.iter_mut().enumerate() {
                    *value = i as u32 * 6;
                }

                output.fill(0);

                unsafe {
                    __aeabi_memcpy4(output.as_mut_ptr(), input.as_ptr(), size * 4);
                }

                for i in 0..size {
                    assert_eq!(input[i], output[i], "Failed with size = {size} at i = {i}");
                }

                for (i, value) in output.iter().enumerate().skip(size) {
                    assert_eq!(*value, 0, "overrun with size = {size} at i = {i}");
                }
            }
        }

        #[test_case]
        fn test_all_of_memcpy(_gba: &mut Gba) {
            let mut input = [0u8; 100];
            let mut output = vec![0u8; 100];

            for size in 0..80 {
                for offset_input in 0..10 {
                    for offset_output in 0..10 {
                        // initialise the buffers
                        for (i, value) in input.iter_mut().enumerate() {
                            *value = i as u8;
                            output[i] = 0;
                        }

                        unsafe {
                            __agbabi_memcpy(
                                output.as_mut_ptr().add(offset_output),
                                input.as_ptr().add(offset_input),
                                size,
                            );
                        }

                        for (i, &v) in output.iter().enumerate() {
                            if i < offset_output {
                                assert_eq!(v, 0, "underrun, size: {size}, input offset: {offset_input}, output offset: {offset_output}, i: {i}");
                            } else if i < offset_output + size {
                                assert_eq!(v, (i - offset_output + offset_input) as u8, "incorrect copy, size: {size}, input offset: {offset_input}, output offset: {offset_output}, i: {i}");
                            } else {
                                assert_eq!(v, 0, "overrun, size: {size}, input offset: {offset_input}, output offset: {offset_output}, i: {i}");
                            }
                        }
                    }
                }
            }
        }

        #[test_case]
        fn test_all_of_memcpy4(_gba: &mut Gba) {
            let mut input = vec![0u32; 100];
            let mut output = vec![0u32; 100];

            for size in 0..80 {
                for offset_input in 0..8 {
                    for offset_output in 0..8 {
                        // initialise the buffers
                        for (i, value) in input.iter_mut().enumerate() {
                            *value = i as u32;
                            output[i] = 0;
                        }

                        unsafe {
                            __aeabi_memcpy4(
                                output.as_mut_ptr().add(offset_output),
                                input.as_ptr().add(offset_input),
                                size * 4,
                            );
                        }

                        for (i, &v) in output.iter().enumerate() {
                            if i < offset_output {
                                assert_eq!(v, 0, "underrun, size: {size}, input offset: {offset_input}, output offset: {offset_output}, i: {i}");
                            } else if i < offset_output + size {
                                assert_eq!(v, (i - offset_output + offset_input) as u32, "incorrect copy, size: {size}, input offset: {offset_input}, output offset: {offset_output}, i: {i}");
                            } else {
                                assert_eq!(v, 0, "overrun, size: {size}, input offset: {offset_input}, output offset: {offset_output}, i: {i}");
                            }
                        }
                    }
                }
            }
        }

        #[test_case]
        fn test_all_of_memcpy4_non_word_length(_gba: &mut Gba) {
            let mut input = vec![0u32; 100];
            let mut output = vec![0u32; 100];

            for size in 0..40 {
                for offset_input in 0..8 {
                    for offset_output in 0..8 {
                        // initialise the buffers
                        for (i, value) in input.iter_mut().enumerate() {
                            *value = i as u32;
                            output[i] = 0;
                        }

                        unsafe {
                            __aeabi_memcpy4(
                                output.as_mut_ptr().add(offset_output),
                                input.as_ptr().add(offset_input),
                                size * 4,
                            );
                        }

                        let input_bytes: &[u8] = unsafe {
                            slice::from_raw_parts(input.as_ptr().cast(), input.len() * 4)
                        };
                        let output_bytes: &[u8] = unsafe {
                            slice::from_raw_parts(output.as_ptr().cast(), output.len() * 4)
                        };

                        for i in 0..input_bytes.len() {
                            if i < offset_output * 4 {
                                assert_eq!(output_bytes[i], 0, "underrun, size: {size}, input offset: {offset_input}, output offset: {offset_output}, i: {i}");
                            } else if i < offset_output * 4 + size * 4 {
                                assert_eq!(output_bytes[i], input_bytes[i - offset_output * 4 + offset_input * 4], "incorrect copy, size: {size}, input offset: {offset_input}, output offset: {offset_output}, i: {i}");
                            } else {
                                assert_eq!(output_bytes[i], 0, "overrun, size: {size}, input offset: {offset_input}, output offset: {offset_output}, i: {i}");
                            }
                        }
                    }
                }
            }
        }
    }
}
