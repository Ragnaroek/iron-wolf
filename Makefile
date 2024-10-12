## SDL
build-sdl:
	cargo build --release --features sdl

run-sdl:
	cargo run --features sdl

run-sdl-debug:
	cargo run --features sdl -- -goobers

run-sdl-profile:
	sudo -E cargo flamegraph --features sdl --profile=dev -- run

## Web
build-web:
	wasm-pack build --debug --target web --features web

run-web: build-web
	miniserve ./

coverage-sdl:
	cargo tarpaulin --features sdl --ignore-tests --out Lcov

## Misc
test-all:
	cargo test --features sdl
	cargo test --features web

profile:
	sudo -E cargo flamegraph --bench core_loop -- --bench

bench:
	cargo bench --features sdl
