profile:
	sudo -E cargo flamegraph --bench core_loop -- --bench

build-sdl:
	cargo build --release --features sdl

run-sdl:
	cargo run --features sdl

run-sdl-debug:
	cargo run --features sdl -- -goobers

run-sdl-profile:
	sudo -E cargo flamegraph --features sdl --profile=dev -- run 

build-web:
	wasm-pack build --debug --target web --features web

run-web: build-web
	miniserve ./

coverage-sdl:
	cargo tarpaulin --features sdl --ignore-tests --out Lcov

test:
	cargo test --features sdl
	cargo test --features web

bench:
	cargo bench --features sdl