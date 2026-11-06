# How to run

You'll need the Rust toolchain with the wasm target, the `wasm-bindgen` CLI at the version
pinned in `Cargo.toml`, and [just](https://github.com/casey/just):

```
git clone --recurse-submodules https://github.com/kshitijl/tinyrenderer-wasm.git
cd tinyrenderer-wasm
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129
just serve
```

Then open http://localhost:8000 and press start.

`just check` runs `cargo fmt --check` and clippy. `cargo` builds for wasm by default here
(`.cargo/config.toml`), which also tells getrandom to use the browser's crypto.

`tools/sync-to-site.sh` builds and copies `web/` to the klauria.tech repo's
`public/demos/tinyrenderer-rs/`, where it's embedded in the tinyrenderer-rs page.

To use a newer tinyrenderer-rs, `git -C tinyrenderer-rs pull` and commit the submodule.
