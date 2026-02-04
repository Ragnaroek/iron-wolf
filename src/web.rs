extern crate web_sys;

use std::cell::RefCell;
use std::io::Cursor;
use std::rc::Rc;

use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::Window;

use crate::assets::{self};
use crate::config;
use crate::gamedata;
use crate::loader::Loader;
use crate::map;
use crate::start::iw_start;

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
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub async fn iw_init(upload_id: &str) {
    console_error_panic_hook::set_once();

    register_upload_loader(upload_id);

    let mut shareware_loader = Loader::new_empty(&assets::W3D1);
    load_shareware_data(&mut shareware_loader)
        .await
        .expect("load shareware data");
    iw_start_web(shareware_loader).expect("iw_start_web failed");
}

pub async fn load_shareware_data(loader: &mut Loader) -> Result<(), JsValue> {
    let win = web_sys::window().unwrap();

    let file_name = loader.file_name(assets::GRAPHIC_DICT);
    let data = load_shareware_file(&file_name, &win).await?;
    loader.load(file_name, data);

    let file_name = loader.file_name(assets::GRAPHIC_HEAD);
    let data = load_shareware_file(&file_name, &win).await?;
    loader.load(file_name, data);

    let file_name = loader.file_name(assets::GRAPHIC_DATA);
    let data = load_shareware_file(&file_name, &win).await?;
    loader.load(file_name, data);

    let file_name = loader.file_name(assets::MAP_HEAD);
    let data = load_shareware_file(&file_name, &win).await?;
    loader.load(file_name, data);

    let file_name = loader.file_name(assets::GAME_MAPS);
    let data = load_shareware_file(&file_name, &win).await?;
    loader.load(file_name, data);

    let file_name = loader.file_name(assets::GAMEDATA);
    let data = load_shareware_file(&file_name, &win).await?;
    loader.load(file_name, data);

    let file_name = loader.file_name(assets::CONFIG_DATA);
    let data = load_shareware_file(&file_name, &win).await?;
    loader.load(file_name, data);

    let file_name = loader.file_name(assets::AUDIO_HEAD);
    let data = load_shareware_file(&file_name, &win).await?;
    loader.load(file_name, data);

    let file_name = loader.file_name(assets::AUDIO_DATA);
    let data = load_shareware_file(&file_name, &win).await?;
    loader.load(file_name, data);

    Ok(())
}

async fn load_shareware_file(file_name: &str, win: &Window) -> Result<Vec<u8>, JsValue> {
    let resp_value =
        JsFuture::from(win.fetch_with_str(&format!("shareware/{}", file_name))).await?;
    let resp: web_sys::Response = resp_value.dyn_into()?;
    let buffer = JsFuture::from(resp.array_buffer()?).await?;

    let array = Uint8Array::new(&buffer);
    Ok(array.to_vec())
}

#[wasm_bindgen]
pub fn iw_start_web(loader: Loader) -> Result<(), String> {
    let iw_config = config::default_iw_config()?;
    iw_start(loader, iw_config)
}

fn register_upload_loader(id: &str) {
    let loader = Loader::new_empty(&assets::W3D6);
    let loader_ref = Rc::new(RefCell::new(loader));

    let document = web_sys::window().unwrap().document().unwrap();
    let button_elem = document
        .get_element_by_id(id)
        .expect("upload button not found");
    let button = button_elem
        .dyn_into::<web_sys::HtmlInputElement>()
        .expect("wrong input element");
    let click_handler: Closure<dyn FnMut(_)> =
        Closure::once(move |e: web_sys::Event| handle_upload(e, loader_ref));

    button
        .add_event_listener_with_callback("change", click_handler.as_ref().unchecked_ref())
        .expect("add event");
    click_handler.forget();
}

fn handle_upload(event: web_sys::Event, loader: Rc<RefCell<Loader>>) {
    let input = event
        .target()
        .expect("upload button target")
        .dyn_into::<web_sys::HtmlInputElement>()
        .expect("input element");
    let files = input.files().expect("files");

    for i in 0..files.length() {
        let file = files.get(i).expect("file");
        let reader = web_sys::FileReader::new().expect("FileReader");
        reader.read_as_array_buffer(&file).expect("read triggered");
        let name = file.name();
        let handle_ref = loader.clone();
        let load_handler: Closure<dyn FnMut(_)> =
            Closure::once(move |e: web_sys::Event| handle_load(e, name, handle_ref));
        reader
            .add_event_listener_with_callback("loadend", load_handler.as_ref().unchecked_ref())
            .expect("add event");
        load_handler.forget();
    }
}

fn handle_load(event: web_sys::Event, name: String, loader: Rc<RefCell<Loader>>) {
    let reader = event
        .target()
        .expect("reader target")
        .dyn_into::<web_sys::FileReader>()
        .expect("file reader");
    let vec_data = js_sys::Uint8Array::new(&reader.result().expect("buffer")).to_vec();
    let all_loaded = {
        let mut l = loader.borrow_mut();
        l.load(name.to_string(), vec_data);
        l.all_files_loaded()
    };

    if all_loaded {
        let l = Rc::<RefCell<Loader>>::try_unwrap(loader)
            .unwrap()
            .into_inner();
        iw_start_web(l).expect("iw start");
    }
}

// Assets

#[wasm_bindgen]
pub fn gamepal_color(ix: usize) -> JsValue {
    let result = assets::gamepal_color(ix);
    serde_wasm_bindgen::to_value(&result).expect("serialise gamepal_color")
}

// Gamedata

#[wasm_bindgen]
pub fn load_gamedata_headers(buffer: &Buffer) -> JsValue {
    let bytes: Vec<u8> = js_sys::Uint8Array::new_with_byte_offset_and_length(
        &buffer.buffer(),
        buffer.byte_offset(),
        buffer.length(),
    )
    .to_vec();

    let result = gamedata::load_gamedata_headers(&bytes).unwrap();
    serde_wasm_bindgen::to_value(&result).expect("serialize gamedata headers")
}

#[wasm_bindgen]
pub fn load_texture(gamedata_js: &Buffer, header_js: JsValue) -> JsValue {
    let gamedata: Vec<u8> = js_sys::Uint8Array::new_with_byte_offset_and_length(
        &gamedata_js.buffer(),
        gamedata_js.byte_offset(),
        gamedata_js.length(),
    )
    .to_vec();

    let header: gamedata::GamedataHeader =
        serde_wasm_bindgen::from_value(header_js).expect("deserialize header");
    let result_load = gamedata::load_texture(&mut Cursor::new(gamedata), &header);
    if result_load.is_err() {
        println!("result = {:?}", result_load.as_ref().err());
    } else {
        println!("texture load ok");
    }
    let result = result_load.expect("load texture");
    serde_wasm_bindgen::to_value(&result).expect("deserialize texture")
}

// Map

#[wasm_bindgen]
pub fn load_map(
    map_data_js: &Buffer,
    map_headers_js: JsValue,
    map_offsets_js: JsValue,
    mapnum: usize,
) -> JsValue {
    let map_data: Vec<u8> = js_sys::Uint8Array::new_with_byte_offset_and_length(
        &map_data_js.buffer(),
        map_data_js.byte_offset(),
        map_data_js.length(),
    )
    .to_vec();

    let map_headers: Vec<map::MapType> =
        serde_wasm_bindgen::from_value(map_headers_js).expect("deserialise maptype");
    let map_offsets: map::MapFileType =
        serde_wasm_bindgen::from_value(map_offsets_js).expect("deserialise mapfiletype");
    let result = map::load_map(
        &mut Cursor::new(map_data),
        &map_headers,
        &map_offsets,
        mapnum,
    )
    .unwrap();
    serde_wasm_bindgen::to_value(&result).expect("serialise mapsegs")
}

#[wasm_bindgen]
pub fn load_map_offsets(buffer: &Buffer) -> JsValue {
    let bytes: Vec<u8> = js_sys::Uint8Array::new_with_byte_offset_and_length(
        &buffer.buffer(),
        buffer.byte_offset(),
        buffer.length(),
    )
    .to_vec();

    let result = map::load_map_offsets(&bytes).unwrap();
    serde_wasm_bindgen::to_value(&result).expect("serialise mapfiletype")
}

#[wasm_bindgen]
pub fn load_map_headers(buffer: &Buffer, offsets_js: JsValue) -> JsValue {
    let bytes: Vec<u8> = js_sys::Uint8Array::new_with_byte_offset_and_length(
        &buffer.buffer(),
        buffer.byte_offset(),
        buffer.length(),
    )
    .to_vec();

    let offsets: map::MapFileType =
        serde_wasm_bindgen::from_value(offsets_js).expect("deserialise mapfiletype");
    let (_, result) = map::load_map_headers(&bytes, offsets).unwrap();
    serde_wasm_bindgen::to_value(&result).expect("serialise maptype")
}
