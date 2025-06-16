# SDL
build-sdl:
    cargo build --release --features sdl

build-sdl-tracing:
    cargo build --release --features sdl,tracing

run-sdl:
    RUST_BACKTRACE=1 cargo run --features sdl -- -goobers

run-sdl-shareware:
    RUST_BACKTRACE=1 cargo run --features sdl -- -config ./shareware_config.toml

run-sdl-tracing:
    cargo run --features sdl,tracing

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
	RUST_BACKTRACE=1 cargo test --features sdl

test-web:
	RUST_BACKTRACE=1 cargo test --features web

test-all: build-sdl-tracing test-sdl test-web


## Misc
profile:
	sudo -E cargo flamegraph --bench core_loop -- --bench

bench:
	cargo bench --features sdl
