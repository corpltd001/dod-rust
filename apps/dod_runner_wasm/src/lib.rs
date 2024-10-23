use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
// #[cfg(feature = "wee_alloc")]
// #[global_allocator]
// static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


#[wasm_bindgen]
pub fn dod_runner(_nonce: u32, _time: u32, hashes: u64, tx: Vec<u8>, offset: usize, prefix: String, ext: Option<String>) -> Result<u64, String> {
    dod_utils::mine::mine_bitwork(tx, offset, hashes, prefix, ext)
}