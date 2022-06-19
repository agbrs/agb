#[cfg(test)]
mod test {
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
                    *value = i as u8 * 6;
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
    }
}
