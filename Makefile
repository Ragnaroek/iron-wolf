## SDL
build-sdl:
	cargo build --release --features sdl

build-sdl-tracing:
	cargo build --release --features sdl,tracing

run-sdl:
	cargo run --features sdl

run-sdl-tracing:
		cargo run --features sdl,tracing

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

## Testing

test-sdl:
	cargo test --features sdl

test-web:
	cargo test --features web

test-all: build-sdl-tracing test-sdl test-web


## Misc
profile:
	sudo -E cargo flamegraph --bench core_loop -- --bench

bench:
	cargo bench --features sdl
