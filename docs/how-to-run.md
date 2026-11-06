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

## How it works

[tinyrenderer-rs](https://github.com/kshitijl/tinyrenderer-rs) was written without LLMs. This
repo, written with one (Claude), compiles its source for the browser without touching its
history: the original is a git submodule (`tinyrenderer-rs/`), and its renderer and game source
files are compiled as they are.

`README.md` is the owner's own words: they want no LLM prose in any of their GitHub READMEs
(Nov 5), so the docs are here.

- `src/lib.rs` includes tinyrenderer-rs's `game`, `render`, `image`, `mesh` and `audio` modules
  from the submodule with `#[path]`, and supplies what its native `main.rs` would: the `Args`
  of its `just launch`, and a frame loop driven from JavaScript instead of winit and pixels.
- There is no filesystem in the browser, so the `.obj` files are built into the wasm binary.
  tinyrenderer-rs's `Mesh::from_file` asks for them with `crate::read_asset` when compiled for
  wasm (its only change for this, made by hand in that repo).
- `web/index.html` runs the loop: each frame the game draws into an RGBA buffer, which is copied
  onto a 320 x 320 canvas scaled up to the page's width. Keys go to the game by their
  `KeyboardEvent.code`, which uses the same names as winit's `KeyCode`. The start button, and
  then a click on the game, lock the pointer for mouse look, where the browser has pointer
  lock: iPhone Safari has none, and calling it there threw and stopped start before the wasm
  loaded.
- On a touchscreen with no mouse, touch controls appear below the game once it starts. They
  press the game's keys and move it like the mouse, so the game itself is unchanged. Arrows
  hold W, A, S and D while touched. Examine is F, which the game wants held; the owner asked
  for it to be a toggle instead (Nov 6), so a tap holds it and the next lets go. The owner also
  asked for only wireframe (1) and split screen (5), labelled by name rather than number; each
  tap presses the key once. Dragging on the game turns the camera, scaled so a drag across its
  width turns about 90 degrees (the mouse's 1/1000 radian per pixel would make that about 20
  on a phone).
- Each frame advances the game in equal steps of at most 5 ms, and by at most 250 ms in all.
  The game moves things by speed times the step and checks only where they land, so long steps
  (a hidden tab's next frame covers the whole gap; 60 fps is already too long, rarely) let the
  player through the maze's outer wall, and then a guard that chased them out panics looking up
  its cell. The numbers are in `src/lib.rs`, by `MAX_STEP`.
- The wasm loads when start is pressed.
- The bytebeat music is tinyrenderer-rs's `audio.rs` as it is, on cpal's Web Audio backend. It
  starts muted: the page replaces the browser's `AudioContext` with a subclass that stays
  suspended until the unmute button, whatever cpal asks. So the bindings are imported on start,
  after the swap: they keep the `AudioContext` there is when they load.
- Clippy lints that tinyrenderer-rs's code trips are allowed on those modules here
  (`src/lib.rs`), rather than fixed there.
