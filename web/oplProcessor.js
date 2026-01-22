class OPLProcessor extends AudioWorkletProcessor {
  constructor(options) {
    super();
    this.imf_data_ptr = 0;
    this.imf_data_len = 0;
    this.adl_data_ptr = 0;
    this.adl_data_len = 0;
    this.adl_playing = false;

    const { wasmBytes, mixerRate, imfClockRate, adlClockRate } = options.processorOptions;
    const module = new WebAssembly.Module(wasmBytes);
    const instance = new WebAssembly.Instance(module, {});
    this.wasm = instance.exports;

    this.generatorPtr = this.wasm.new_generator(mixerRate, imfClockRate, adlClockRate);

    this.port.onmessage = (event) => {
      if (event.data.cmd === "play_imf") {
        if (this.imf_data_ptr) {
          this.wasm.dealloc(this.imf_data_ptr, this.imf_data_len);
        }

        let bytes = event.data.data;
        this.imf_data_len = bytes.length;
        this.imf_data_ptr = this.wasm.alloc(this.imf_data_len);

        let ptr_bytes = new Uint8Array(this.wasm.memory.buffer, this.imf_data_ptr, this.imf_data_len);
        ptr_bytes.set(bytes);
        this.wasm.play_imf(this.generatorPtr, this.imf_data_ptr, this.imf_data_len);
      } else if (event.data.cmd === "play_adl") {
        if (this.adl_data_ptr) {
          this.wasm.dealloc(this.adl_data_ptr, this.adl_data_len);
        }
        let bytes = event.data.data;
        this.adl_data_len = bytes.length;
        this.adl_data_ptr = this.wasm.alloc(this.adl_data_len);
        let ptr_bytes = new Uint8Array(this.wasm.memory.buffer, this.adl_data_ptr, this.adl_data_len);
        ptr_bytes.set(bytes);
        this.wasm.play_adl(this.generatorPtr, this.adl_data_ptr, this.adl_data_len);
        this.adl_playing = true;
      } else if (event.data.cmd === "write_reg") {
        this.wasm.write_reg(event.data.reg, event.data.value);
      } else if (event.data.cmd === "stop_imf") {
        this.wasm.stop_imf();
      }
    };
  }

  process(inputs, outputs) {
    const ptr = this.wasm.generate_block(this.generatorPtr);
    const bytes = new Float32Array(this.wasm.memory.buffer, ptr, 256);

    const output = outputs[0];
    for (let i = 0; i < 128; i++) {
      output[0][i] = bytes[i * 2];
      output[1][i] = bytes[i * 2 + 1];
    }

    const adl_playing = this.wasm.is_adl_playing(this.generatorPtr);
    if (this.adl_playing && !adl_playing) {
      this.adl_playing = false;
      //notify on "falling flank" to the main thread
      this.port.postMessage("adl_finished");
    }

    return true; // keep processor alive
  }
}

registerProcessor("opl-processor", OPLProcessor);
