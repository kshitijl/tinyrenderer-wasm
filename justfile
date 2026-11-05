# Build the wasm and its JavaScript bindings into web/pkg.
build:
    cargo build --release
    wasm-bindgen --target web --out-dir web/pkg target/wasm32-unknown-unknown/release/tinyrenderer_wasm.wasm

# Build, then serve the page at http://localhost:8000.
serve: build
    python3 -m http.server --directory web 8000

check:
    cargo fmt --check
    cargo clippy --release --all-targets -- -D warnings
