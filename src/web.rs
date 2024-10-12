extern crate web_sys;

use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;
use std::io::Cursor;

use wasm_bindgen::prelude::*;

use crate::start::iw_start;
use crate::assets::{self, WolfFile, WolfVariant, file_name};
use crate::loader::Loader;
use crate::config;
use crate::map;
use crate::gamedata;
use crate::patch::PatchConfig;

#[wasm_bindgen]
pub fn iw_init(upload_id: &str) {
    console_error_panic_hook::set_once(); 

    let loader = WebLoader::new();
    register_loader(upload_id, loader);
}

#[wasm_bindgen]
pub fn iw_start_web(loader: &WebLoader) -> Result<(), String> {
    let iw_config = config::default_iw_config();
    iw_start(loader, iw_config)
}

// WebLoader

#[wasm_bindgen]
extern "C" {
    pub type Buffer;

    #[wasm_bindgen(method, getter)]
    pub fn buffer(this: &Buffer) -> js_sys::ArrayBuffer;

    #[wasm_bindgen(method, getter, js_name = byteOffset)]
    pub fn byte_offset(this: &Buffer) -> u32;

    #[wasm_bindgen(method, getter)]
    pub fn length(this: &Buffer) -> u32;
}

#[wasm_bindgen]
pub struct WebLoader {
    files: HashMap<String, Vec<u8>>,
}

#[wasm_bindgen]
impl WebLoader {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WebLoader {
        WebLoader{
            files : HashMap::new(),
        }
    }

    pub fn load(&mut self, file: String, data: Vec<u8>) {
        self.files.insert(file, data);
    }

    pub fn all_files_loaded(&self) -> bool {
        self.files.contains_key(assets::GRAPHIC_DICT) 
        && self.files.contains_key(assets::GRAPHIC_HEAD) 
        && self.files.contains_key(assets::GRAPHIC_DATA)
        && self.files.contains_key(assets::MAP_HEAD)
        && self.files.contains_key(assets::GAME_MAPS)
        && self.files.contains_key(assets::GAMEDATA)
        && self.files.contains_key(assets::CONFIG_DATA)
    }
}

#[wasm_bindgen]
pub fn register_loader(id: &str, loader: WebLoader) {
    let document = web_sys::window().unwrap().document().unwrap();
    let button_elem = document.get_element_by_id(id).expect("upload button not found");
    let button = button_elem.dyn_into::<web_sys::HtmlInputElement>().expect("wrong input element");
    let click_handler : Closure::<dyn FnMut(_)> = Closure::once(move |e: web_sys::Event| handle_upload(e, loader));    
    
    button.add_event_listener_with_callback("change", click_handler.as_ref().unchecked_ref()).expect("add event");
    click_handler.forget();
}

fn handle_upload(event: web_sys::Event, loader: WebLoader) {
    let input = event.target().expect("upload button target").dyn_into::<web_sys::HtmlInputElement>().expect("input element");
    let files = input.files().expect("files");

    let loader_ref = Rc::new(RefCell::new(loader));
    for i in 0..files.length() {
        let file = files.get(i).expect("file");
        let reader = web_sys::FileReader::new().expect("FileReader");
        reader.read_as_array_buffer(&file).expect("read triggered");
        let name = file.name();
        let handle_ref = loader_ref.clone();
        let load_handler : Closure::<dyn FnMut(_)> = Closure::once(move |e:web_sys::Event| handle_load(e, name, handle_ref));
        reader.add_event_listener_with_callback("loadend", load_handler.as_ref().unchecked_ref()).expect("add event");
        load_handler.forget();
    }
}

fn handle_load(event: web_sys::Event, name: String, loader: Rc<RefCell<WebLoader>>) {
    let reader = event.target().expect("reader target").dyn_into::<web_sys::FileReader>().expect("file reader");
    let vec_data = js_sys::Uint8Array::new(&reader.result().expect("buffer")).to_vec();
    {
        loader.borrow_mut().load(name.to_string(), vec_data);
    }
    
    let loader_borrow = &loader.borrow();
    if loader_borrow.all_files_loaded() {
        iw_start_web(loader_borrow).expect("iw start");
    }
}

impl Loader for WebLoader {
    fn load_wolf_file(&self, file: WolfFile, variant: &WolfVariant) -> Vec<u8> {
        let buffer = self.files.get(&file_name(file, variant)).expect(&format!("file {} not found", file_name(file, variant)));
        buffer.clone()
    }

    fn load_patch_config_file(&self) -> Option<PatchConfig> {
        todo!("patch file loading not implemented for web");
    }
    // panics, if patch path is not set
    fn load_patch_data_file(&self, name: String) -> Vec<u8> {
        todo!("patch file data loading not implemented for web");
    }
}

// Assets

#[wasm_bindgen]
pub fn gamepal_color(ix: usize) -> JsValue {
	let result = assets::gamepal_color(ix);
    JsValue::from_serde(&result).unwrap()
}

// Gamedata

#[wasm_bindgen]
pub fn load_gamedata_headers(buffer: &Buffer) -> JsValue {
    let bytes: Vec<u8> = js_sys::Uint8Array::new_with_byte_offset_and_length(
        &buffer.buffer(),
        buffer.byte_offset(),
        buffer.length(),
    ).to_vec();

	let result = gamedata::load_gamedata_headers(&bytes).unwrap();
	JsValue::from_serde(&result).unwrap()
}

#[wasm_bindgen]
pub fn load_texture(gamedata_js: &Buffer, header_js: &JsValue) -> JsValue {
    let gamedata: Vec<u8> = js_sys::Uint8Array::new_with_byte_offset_and_length(
        &gamedata_js.buffer(),
        gamedata_js.byte_offset(),
        gamedata_js.length(),
    ).to_vec();

	let header : gamedata::GamedataHeader = header_js.into_serde().unwrap();
	let result = gamedata::load_texture(&mut Cursor::new(gamedata), &header).unwrap();
	JsValue::from_serde(&result).unwrap()
}


// Map

#[wasm_bindgen]
pub fn load_map(map_data_js: &Buffer, map_headers_js: &JsValue, map_offsets_js: &JsValue, mapnum: usize) -> JsValue {
    let map_data: Vec<u8> = js_sys::Uint8Array::new_with_byte_offset_and_length(
        &map_data_js.buffer(),
        map_data_js.byte_offset(),
        map_data_js.length(),
    ).to_vec();

	let map_headers : Vec<map::MapType> = map_headers_js.into_serde().unwrap();
	let map_offsets : map::MapFileType = map_offsets_js.into_serde().unwrap();
	let result = map::load_map(&mut Cursor::new(map_data), &map_headers, &map_offsets, mapnum).unwrap();
	JsValue::from_serde(&result).unwrap()
}

#[wasm_bindgen]
pub fn load_map_offsets(buffer: &Buffer) -> JsValue {
    let bytes: Vec<u8> = js_sys::Uint8Array::new_with_byte_offset_and_length(
        &buffer.buffer(),
        buffer.byte_offset(),
        buffer.length(),
    ).to_vec();

	let result = map::load_map_offsets(&bytes).unwrap();
	JsValue::from_serde(&result).unwrap()
}

#[wasm_bindgen]
pub fn load_map_headers(buffer: &Buffer, offsets_js: &JsValue) -> JsValue {
	let bytes: Vec<u8> = js_sys::Uint8Array::new_with_byte_offset_and_length(
        &buffer.buffer(),
        buffer.byte_offset(),
        buffer.length(),
    ).to_vec();

	let offsets: map::MapFileType = offsets_js.into_serde().unwrap();
	let (_, result) = map::load_map_headers(&bytes, offsets).unwrap();
	JsValue::from_serde(&result).unwrap()
}