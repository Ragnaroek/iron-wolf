use js_sys::Uint8Array;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::assets::{self, WolfFile, WolfVariant, file_name};
use crate::patch::PatchConfig;

const PATCH_FILE_NAME: &'static str = "patch.toml";
const INDEXED_DB_NAME: &'static str = "iron-wolf";
const INDEXED_DB_SAVE_STORE: &'static str = "saves";

#[wasm_bindgen]
#[derive(Debug)]
pub struct Loader {
    variant: &'static WolfVariant,
    files: HashMap<String, Vec<u8>>,
}

impl Loader {
    pub fn new_empty(variant: &'static WolfVariant) -> Loader {
        Loader {
            variant,
            files: HashMap::new(),
        }
    }
}

impl Loader {
    pub fn new_shareware() -> Loader {
        Loader {
            variant: &assets::W3D1,
            files: HashMap::new(),
        }
    }

    pub fn variant(&self) -> &'static WolfVariant {
        return self.variant;
    }

    pub fn write_wolf_file(&self, _file: WolfFile, _data: &[u8]) -> Result<(), String> {
        todo!("write_wolf_file not implemented for web");
    }

    pub fn load_wolf_file(&self, file: WolfFile) -> Vec<u8> {
        let buffer = self
            .files
            .get(&file_name(file, &self.variant))
            .expect(&format!(
                "file {} not found",
                file_name(file, &self.variant)
            ));
        buffer.clone()
    }

    pub fn load_patch_config_file(&self) -> Result<Option<PatchConfig>, String> {
        if let Some(bytes) = self.files.get(PATCH_FILE_NAME) {
            let config: PatchConfig = toml::from_slice(&bytes).map_err(|e| e.to_string())?;
            Ok(Some(config))
        } else {
            Ok(None)
        }
    }
    // panics, if patch path is not set
    pub fn load_patch_data_file(&self, _name: String) -> Vec<u8> {
        todo!("patch file data loading not implemented for web");
    }

    pub async fn load_save_game_head(&self, which: usize) -> Result<Vec<u8>, String> {
        let data = load_savegame_indexeddb(&savegame_name(which))
            .await
            .map_err(|_| "loading savegame")?;
        Ok(data.slice(0, 32).to_vec())
    }

    pub async fn load_save_game(&self, which: usize) -> Result<Vec<u8>, String> {
        let data = load_savegame_indexeddb(&savegame_name(which))
            .await
            .map_err(|_| "loading savegame")?;
        Ok(data.to_vec())
    }

    pub async fn save_save_game(&self, which: usize, bytes: &[u8]) -> Result<(), String> {
        let data = Uint8Array::from(bytes);
        let name = savegame_name(which);
        store_savegame_indexeddb(&name, data)
            .await
            .map_err(|_| "store savegame")?;
        Ok(())
    }

    pub fn load_wolf_file_slice(
        &self,
        file: WolfFile,
        offset_u64: u64,
        len: usize,
    ) -> Result<Vec<u8>, String> {
        let buffer = self
            .files
            .get(&file_name(file, &self.variant))
            .expect(&format!(
                "file {} not found",
                file_name(file, &self.variant)
            ));
        let offset = offset_u64 as usize;
        Ok(buffer[offset..(offset + len)].to_vec())
    }

    pub fn file_name(&self, asset_name: &str) -> String {
        format!("{}.{}", asset_name, self.variant.file_ending)
    }

    // helper

    pub fn load(&mut self, file: String, data: Vec<u8>) {
        self.files.insert(file, data);
    }

    pub fn all_files_loaded(&self) -> bool {
        self.files
            .contains_key(&self.file_name(assets::GRAPHIC_DICT))
            && self
                .files
                .contains_key(&self.file_name(assets::GRAPHIC_HEAD))
            && self
                .files
                .contains_key(&self.file_name(assets::GRAPHIC_DATA))
            && self.files.contains_key(&self.file_name(assets::MAP_HEAD))
            && self.files.contains_key(&self.file_name(assets::GAME_MAPS))
            && self.files.contains_key(&self.file_name(assets::GAMEDATA))
            && self
                .files
                .contains_key(&self.file_name(assets::CONFIG_DATA))
            && self.files.contains_key(&self.file_name(assets::AUDIO_HEAD))
            && self.files.contains_key(&self.file_name(assets::AUDIO_DATA))
    }
}

fn savegame_name(which: usize) -> String {
    format!("SAVEGAM{}", which)
}

async fn store_savegame_indexeddb(save_name: &str, data: Uint8Array) -> Result<(), JsValue> {
    let db = open_db().await?;
    let transaction = db.transaction_with_str_and_mode(
        INDEXED_DB_SAVE_STORE,
        web_sys::IdbTransactionMode::Readwrite,
    )?;

    let store = transaction.object_store(INDEXED_DB_SAVE_STORE)?;
    idb_request_await(&store.put_with_key(&data, &save_name.into())?)
        .await
        .map_err(|_| "idb store failed")?;
    Ok(())
}

async fn load_savegame_indexeddb(save_name: &str) -> Result<Uint8Array, JsValue> {
    let db = open_db().await?;
    let transaction = db.transaction_with_str_and_mode(
        INDEXED_DB_SAVE_STORE,
        web_sys::IdbTransactionMode::Readwrite,
    )?;

    let store = transaction.object_store(INDEXED_DB_SAVE_STORE)?;
    let value = idb_request_await(&store.get(&save_name.into())?)
        .await
        .map_err(|_| "idb load failed")?;
    if value.is_undefined() {
        Err(JsValue::NULL)
    } else {
        let uint8_array = Uint8Array::new(&value);
        Ok(uint8_array)
    }
}

async fn open_db() -> Result<web_sys::IdbDatabase, JsValue> {
    let window = web_sys::window().expect("global window access");
    let factory = window.indexed_db().map_err(|e| e)?;
    if let Some(factory) = factory {
        let open_request = factory.open_with_u32(INDEXED_DB_NAME, 2)?;

        let db_promise = js_sys::Promise::new(&mut |resolve, reject| {
            let onsuccess = Closure::wrap(Box::new(move |event: web_sys::Event| {
                let db = web_sys::IdbDatabase::from(
                    js_sys::Reflect::get(&event, &JsValue::from_str("target"))
                        .unwrap()
                        .dyn_into::<web_sys::IdbOpenDbRequest>()
                        .unwrap()
                        .result()
                        .unwrap(),
                );
                resolve.call1(&JsValue::NULL, &db).unwrap();
            }) as Box<dyn FnMut(_)>);

            let onerror = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                let error = "opening IndexDB failed".into();
                reject.call1(&JsValue::NULL, &error).unwrap();
            }) as Box<dyn FnMut(_)>);

            let onupgradeneeded = Closure::wrap(Box::new(move |event: web_sys::Event| {
                let db = web_sys::IdbDatabase::from(
                    js_sys::Reflect::get(&event, &JsValue::from_str("target"))
                        .unwrap()
                        .dyn_into::<web_sys::IdbOpenDbRequest>()
                        .unwrap()
                        .result()
                        .unwrap(),
                );
                if !db.object_store_names().contains(INDEXED_DB_SAVE_STORE) {
                    db.create_object_store(INDEXED_DB_SAVE_STORE)
                        .expect("created save store");
                }
            }) as Box<dyn FnMut(_)>);

            open_request.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
            open_request.set_onerror(Some(onerror.as_ref().unchecked_ref()));
            open_request.set_onupgradeneeded(Some(onupgradeneeded.as_ref().unchecked_ref()));
            onsuccess.forget();
            onerror.forget();
            onupgradeneeded.forget();
        });

        let db = JsFuture::from(db_promise).await?;
        let db = web_sys::IdbDatabase::from(db);
        Ok(db)
    } else {
        Err("could not access IndexDB".into())
    }
}

async fn idb_request_await(request: &web_sys::IdbRequest) -> Result<JsValue, JsValue> {
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let on_success = Closure::once(move |_: web_sys::Event| {
            resolve.call0(&JsValue::NULL).unwrap();
        });
        let on_error = Closure::once(move |e: JsValue| {
            reject.call1(&JsValue::NULL, &e).unwrap();
        });

        request.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
        request.set_onerror(Some(on_error.as_ref().unchecked_ref()));

        on_success.forget();
        on_error.forget();
    });
    JsFuture::from(promise).await?;
    request.result()
}
