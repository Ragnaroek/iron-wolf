use wasm_bindgen::prelude::*;

use super::start::iw_start;

#[wasm_bindgen]
pub fn iw_start_web() -> Result<(), String> {
    console_error_panic_hook::set_once();
    iw_start()
}