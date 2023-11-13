profile:
	sudo -E cargo flamegraph --bench core_loop -- --bench

build-sdl:
	cargo build --features sdl

run-sdl:
	cargo run --features sdl

build-web:
	wasm-pack build --debug --target web --features web