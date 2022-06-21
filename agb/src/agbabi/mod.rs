#[cfg(test)]
mod test {
    mod memset {
        use crate::Gba;
        use alloc::vec;

        extern "C" {
            fn __aeabi_memset(dest: *mut u8, n: usize, v: u8);
            fn __aeabi_memset4(dest: *mut u8, n: usize, v: u8);
        }

        #[test_case]
        fn test_memset_with_different_sizes(_gba: &mut Gba) {
            let mut values = vec![0u8; 100];

            let v = 0x12;

            for n in 0..80 {
                values.fill(0xFF);

                unsafe {
                    __aeabi_memset4(values.as_mut_ptr().wrapping_offset(10), n, v);
                }

                for (i, &v) in values.iter().enumerate().take(10) {
                    assert_eq!(v, 0xFF, "underrun at {}", i);
                }

                for i in 0..n {
                    assert_eq!(values[10 + i], v, "incorrect value at {}", i + 10);
                }

                for (i, &v) in values.iter().enumerate().skip(10 + n) {
                    assert_eq!(v, 0xFF, "overrun at {}", i);
                }
            }
        }
    }

    mod memcpy {
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
        fn test_memcpy_bytes_with_different_sizes(_gba: &mut Gba) {
            let mut input = vec![0u8; 70];
            let mut output = vec![0u8; 70];

            for size in 0..68 {
                for (i, value) in input.iter_mut().enumerate() {
                    *value = i as u8 * 3;
                }

                output.fill(0);

                unsafe {
                    __agbabi_memcpy(output.as_mut_ptr(), input.as_ptr(), size);
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
        fn test_memcpy_bytes_output_offsetted_with_different_sizes(_gba: &mut Gba) {
            let mut input = vec![0u8; 70];
            let mut output = vec![0u8; 70];

            for size in 0..60 {
                for (i, value) in input.iter_mut().enumerate() {
                    *value = i as u8 * 3;
                }

                output.fill(0);

                unsafe {
                    __agbabi_memcpy(output.as_mut_ptr().add(1), input.as_ptr(), size);
                }

                for i in 0..size {
                    assert_eq!(
                        input[i],
                        output[i + 1],
                        "Failed with size = {size} at i = {i}"
                    );
                }

                for (i, value) in output.iter().enumerate().skip(size + 1) {
                    assert_eq!(*value, 0, "overrun with size = {size} at i = {i}");
                }
            }
        }

        #[test_case]
        fn test_memcpy_bytes_input_offsetted_with_different_sizes(_gba: &mut Gba) {
            let mut input = vec![0u8; 70];
            let mut output = vec![0u8; 70];

            for size in 0..60 {
                for (i, value) in input.iter_mut().enumerate() {
                    *value = i as u8 * 3;
                }

                output.fill(0);

                unsafe {
                    __agbabi_memcpy(output.as_mut_ptr(), input.as_ptr().add(1), size);
                }

                for i in 0..size {
                    assert_eq!(
                        input[i + 1],
                        output[i],
                        "Failed with size = {size} at i = {i}"
                    );
                }

                for (i, value) in output.iter().enumerate().skip(size) {
                    assert_eq!(*value, 0, "overrun with size = {size} at i = {i}");
                }
            }
        }

        #[test_case]
        fn test_memcpy_bytes_input_output_offsetted_with_different_sizes(_gba: &mut Gba) {
            let mut input = vec![0u8; 70];
            let mut output = vec![0u8; 70];

            for size in 0..60 {
                for (i, value) in input.iter_mut().enumerate() {
                    *value = i as u8 * 3;
                }

                output.fill(0);

                unsafe {
                    __agbabi_memcpy(output.as_mut_ptr().add(1), input.as_ptr().add(1), size);
                }

                assert_eq!(output[0], 0);

                for i in 1..size + 1 {
                    assert_eq!(input[i], output[i], "Failed with size = {size} at i = {i}");
                }

                for (i, value) in output.iter().enumerate().skip(size + 1) {
                    assert_eq!(*value, 0, "overrun with size = {size} at i = {i}");
                }
            }
        }
    }
}
