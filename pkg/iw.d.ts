/* tslint:disable */
/* eslint-disable */

export class WebLoader {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  all_files_loaded(): boolean;
  load(file: string, data: Uint8Array): void;
}

export function gamepal_color(ix: number): any;

export function init_panic_hook(): void;

export function iw_init(upload_id: string): Promise<void>;

export function iw_start_web(loader: WebLoader): void;

export function load_gamedata_headers(buffer: any): any;

export function load_map(map_data_js: any, map_headers_js: any, map_offsets_js: any, mapnum: number): any;

export function load_map_headers(buffer: any, offsets_js: any): any;

export function load_map_offsets(buffer: any): any;

export function load_texture(gamedata_js: any, header_js: any): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_webloader_free: (a: number, b: number) => void;
  readonly gamepal_color: (a: number) => number;
  readonly init_panic_hook: () => void;
  readonly iw_init: (a: number, b: number) => number;
  readonly iw_start_web: (a: number, b: number) => void;
  readonly load_gamedata_headers: (a: number) => number;
  readonly load_map: (a: number, b: number, c: number, d: number) => number;
  readonly load_map_headers: (a: number, b: number) => number;
  readonly load_map_offsets: (a: number) => number;
  readonly load_texture: (a: number, b: number) => number;
  readonly webloader_all_files_loaded: (a: number) => number;
  readonly webloader_load: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly __wasm_bindgen_func_elem_2086: (a: number, b: number, c: number) => void;
  readonly __wasm_bindgen_func_elem_2068: (a: number, b: number) => void;
  readonly __wasm_bindgen_func_elem_472: (a: number, b: number, c: number) => void;
  readonly __wasm_bindgen_func_elem_298: (a: number, b: number) => void;
  readonly __wasm_bindgen_func_elem_2180: (a: number, b: number, c: number) => void;
  readonly __wasm_bindgen_func_elem_2165: (a: number, b: number) => void;
  readonly __wasm_bindgen_func_elem_2821: (a: number, b: number, c: number, d: number) => void;
  readonly __wbindgen_export: (a: number, b: number) => number;
  readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export3: (a: number) => void;
  readonly __wbindgen_export4: (a: number, b: number, c: number) => void;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
