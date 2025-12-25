class OPLProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    this.ringBuffer = [];
    this.ptr = 0; // current offset into the data buffer (always ringBuffer[0])

    this.port.onmessage = (event) => {
      if (event.data instanceof Float32Array) {
        this.ringBuffer.push(event.data);
      } else if (event.data === "CLEAR") {
        this.ringBuffer.length = 0;
      }
    };
  }

  process(inputs, outputs) {
    if (this.ringBuffer.length == 0) {
      return true;
    }

    const output = outputs[0];
    for (let i = 0; i < 128; i++) {
      output[0][i] = this.ringBuffer[0][this.ptr++];
      output[1][i] = this.ringBuffer[0][this.ptr++];
    }

    if (this.ptr >= this.ringBuffer[0].length) {
      this.ringBuffer.shift();
      this.ptr = 0;
    }

    return true; // keep processor alive
  }
}

registerProcessor("opl-processor", OPLProcessor);
