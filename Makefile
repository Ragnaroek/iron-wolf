profile:
	sudo -E cargo flamegraph --bench core_loop -- --bench

build-sdl:
	cargo build --features sdl

run-sdl:
	cargo run --features sdl

run-sdl-debug:
	cargo run --features sdl -- -goobers

build-web:
	wasm-pack build --debug --target web --features web

run-web: build-web
	miniserve ./

coverage-sdl:
	cargo tarpaulin --features sdl --ignore-tests --out Lcov

test:
	cargo test --features sdl
	cargo test --features web