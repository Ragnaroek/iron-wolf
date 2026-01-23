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

run-sdl-demo NUM:
    cargo run --features sdl -- -timedemo {{ NUM }}

# # Web
build-web:
    wasm-pack build --out-dir web/pkg --release --target web --features web

run-web: build-web
    miniserve ./web

coverage-sdl:
    cargo tarpaulin --features sdl --ignore-tests --out Lcov

# # Testing
test:
    RUST_BACKTRACE=1 RUSTFLAGS="-A unused" cargo test --features test

test-all: build-sdl-tracing build-web test

# # Misc
profile:
    sudo -E cargo flamegraph --bench core_loop -- --bench

bench:
    cargo bench --features sdl
