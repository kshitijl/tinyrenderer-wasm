# tinyrenderer-wasm

[tinyrenderer-rs](https://github.com/kshitijl/tinyrenderer-rs) in the browser.

tinyrenderer-rs was written without LLMs. This repo, written with one (Claude), compiles its
source for the browser without touching its history: the original is a git submodule
(`tinyrenderer-rs/`), and its renderer and game source files are compiled as they are.

## Run it

You'll need the Rust toolchain with the wasm target, the `wasm-bindgen` CLI at the version
pinned in `Cargo.toml`, and [just](https://github.com/casey/just):

```
git clone --recurse-submodules <this repo>
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129
just serve
```

Then open http://localhost:8000 and click.

## How it works

- `src/lib.rs` includes tinyrenderer-rs's `game`, `render`, `image`, `mesh` and `audio` modules
  from the submodule with `#[path]`, and supplies what its native `main.rs` would: the `Args`
  of its `just launch`, and a frame loop driven from JavaScript instead of winit and pixels.
- There is no filesystem in the browser, so the `.obj` files are built into the wasm binary.
  tinyrenderer-rs's `Mesh::from_file` asks for them with `crate::read_asset` when compiled for
  wasm (its only change for this, made by hand in that repo).
- `web/index.html` runs the loop: each frame the game draws into an RGBA buffer, which is copied
  onto a 320 x 320 canvas scaled up to fill the window. Keys go to the game by their
  `KeyboardEvent.code`, which uses the same names as winit's `KeyCode`. A click locks the
  pointer for mouse look.
- The bytebeat music is tinyrenderer-rs's `audio.rs` as it is, on cpal's Web Audio backend. The
  game starts on the first click because browsers start audio only after one.
- Clippy lints that tinyrenderer-rs's code trips are allowed on those modules here
  (`src/lib.rs`), rather than fixed there.

`just check` runs `cargo fmt --check` and clippy. `cargo` builds for wasm by default here
(`.cargo/config.toml`), which also tells getrandom to use the browser's crypto.

To use a newer tinyrenderer-rs, `git -C tinyrenderer-rs pull` and commit the submodule.
