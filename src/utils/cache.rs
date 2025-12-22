use brotli::BrotliDecompress;
use std::io::Write;
use tutorlolv2_gen::{BLOCK, BLOCK_SIZE};
use wasm_bindgen::prelude::wasm_bindgen;

pub static mut CACHE: [u8; BLOCK_SIZE] = [0; _];

pub struct FixedBuffer<const N: usize> {
    buffer: &'static mut [u8; N],
    position: usize,
}

impl<const N: usize> Write for FixedBuffer<N> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        unsafe {
            let destination_ptr = self.buffer.as_mut_ptr().add(self.position);
            let source_ptr = buf.as_ptr();
            core::ptr::copy_nonoverlapping(source_ptr, destination_ptr, buf.len());
            self.position += buf.len();
            Ok(buf.len())
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[wasm_bindgen]
pub fn cache_ptr() -> *const u8 {
    unsafe { CACHE.as_ptr() }
}

#[wasm_bindgen]
pub fn cache_len() -> usize {
    BLOCK_SIZE
}

#[cold]
pub fn init_cache() {
    web_sys::console::time();
    unsafe {
        BrotliDecompress(
            &mut (&BLOCK as &[u8]),
            &mut (&mut FixedBuffer {
                buffer: &mut CACHE,
                position: 0,
            }),
        )
        .unwrap_unchecked();
    }
    web_sys::console::time_end();
}
