//! tinyrenderer-rs in the browser.
//!
//! The renderer and the game are tinyrenderer-rs's own source files, compiled
//! unchanged from the submodule. This crate supplies what its native `main.rs`
//! would: `Args`, the `.obj` files (there's no filesystem in the browser) and a
//! frame loop, which here is driven from JavaScript (`web/index.html`).

// Clippy lints that tinyrenderer-rs's code trips. It's compiled as it is, so
// they're allowed here rather than fixed there.
#[allow(
    clippy::manual_checked_ops,
    clippy::manual_is_multiple_of,
    clippy::precedence
)]
#[path = "../tinyrenderer-rs/src"]
mod upstream {
    pub mod audio;
    pub mod game;
    pub mod image;
    pub mod mesh;
    pub mod render;
}

// tinyrenderer-rs's modules refer to each other as `crate::game`, `crate::image`
// and so on, so they have to be reachable from the crate root.
use upstream::{audio, game, image, mesh, render};

use std::time::Duration;
use wasm_bindgen::prelude::*;
use winit::keyboard::KeyCode;

/// The fields of tinyrenderer-rs's command-line arguments that `World::new` reads.
pub struct Args {
    exhibit_models: Vec<String>,
    guard_model: String,
    num_guards: u32,
    wall_model_debug: Option<String>,
    canvas_size: u16,
}

/// The `.obj` files, built into the wasm binary. The upstream `Mesh::from_file`
/// calls this instead of opening a file when compiled for wasm.
const ASSETS: &[(&str, &[u8])] = &[
    (
        "assets/anchor.obj",
        include_bytes!("../tinyrenderer-rs/assets/anchor.obj"),
    ),
    (
        "assets/bat.obj",
        include_bytes!("../tinyrenderer-rs/assets/bat.obj"),
    ),
    (
        "assets/cannon.obj",
        include_bytes!("../tinyrenderer-rs/assets/cannon.obj"),
    ),
    (
        "assets/celestial-globe.obj",
        include_bytes!("../tinyrenderer-rs/assets/celestial-globe.obj"),
    ),
    (
        "assets/diablo.obj",
        include_bytes!("../tinyrenderer-rs/assets/diablo.obj"),
    ),
    (
        "assets/elephant.obj",
        include_bytes!("../tinyrenderer-rs/assets/elephant.obj"),
    ),
    (
        "assets/giraffe.obj",
        include_bytes!("../tinyrenderer-rs/assets/giraffe.obj"),
    ),
    (
        "assets/greek-vase.obj",
        include_bytes!("../tinyrenderer-rs/assets/greek-vase.obj"),
    ),
    (
        "assets/guitar.obj",
        include_bytes!("../tinyrenderer-rs/assets/guitar.obj"),
    ),
    (
        "assets/man.obj",
        include_bytes!("../tinyrenderer-rs/assets/man.obj"),
    ),
    (
        "assets/monstera_plant_medium_potted.obj",
        include_bytes!("../tinyrenderer-rs/assets/monstera_plant_medium_potted.obj"),
    ),
    (
        "assets/rhino.obj",
        include_bytes!("../tinyrenderer-rs/assets/rhino.obj"),
    ),
    (
        "assets/skull.obj",
        include_bytes!("../tinyrenderer-rs/assets/skull.obj"),
    ),
    (
        "assets/toilet.obj",
        include_bytes!("../tinyrenderer-rs/assets/toilet.obj"),
    ),
    (
        "assets/walrus.obj",
        include_bytes!("../tinyrenderer-rs/assets/walrus.obj"),
    ),
];

pub fn read_asset(filename: &str) -> std::io::Result<&'static [u8]> {
    ASSETS
        .iter()
        .find(|(name, _)| *name == filename)
        .map(|(_, contents)| *contents)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, filename.to_string()))
}

/// The settings of tinyrenderer-rs's `just launch`.
fn launch_args() -> Args {
    let exhibits = [
        "cannon",
        "rhino",
        "skull",
        "toilet",
        "diablo",
        "giraffe",
        "greek-vase",
        "guitar",
        "monstera_plant_medium_potted",
        "bat",
        "celestial-globe",
        "elephant",
        "anchor",
        "walrus",
    ];
    Args {
        exhibit_models: exhibits.iter().map(|e| format!("assets/{e}.obj")).collect(),
        guard_model: "assets/man.obj".to_string(),
        num_guards: 8,
        wall_model_debug: None,
        canvas_size: 320,
    }
}

/// The keys the game reads, by their names in JavaScript's `KeyboardEvent.code`.
fn key_code(code: &str) -> Option<KeyCode> {
    match code {
        "KeyW" => Some(KeyCode::KeyW),
        "KeyA" => Some(KeyCode::KeyA),
        "KeyS" => Some(KeyCode::KeyS),
        "KeyD" => Some(KeyCode::KeyD),
        "KeyF" => Some(KeyCode::KeyF),
        "Digit1" => Some(KeyCode::Digit1),
        "Digit4" => Some(KeyCode::Digit4),
        "Digit5" => Some(KeyCode::Digit5),
        "Digit6" => Some(KeyCode::Digit6),
        _ => None,
    }
}

/// The game and its RGBA frame, which JavaScript copies onto a canvas.
#[wasm_bindgen]
pub struct Game {
    world: game::World,
    frame: Vec<u8>,
    frame_width: u32,
    frame_height: u32,
    /// Time in the game: the sum of the steps it has been advanced by.
    game_time: Duration,
}

/// The longest step the game is advanced by at once. The game moves things by speed times the
/// step and checks only where they land, so long steps let the player through the maze's outer
/// wall, after which guards that chased them out panic looking up their cell off the map. Measured
/// in a headless native run walking at random for 10 minutes, 24 games each: steps of 8 ms never
/// got out, 16 ms (60 fps) did once, 33 ms 10 times; in steps of at most 5 ms, frames of 16 to
/// 250 ms never did.
const MAX_STEP: Duration = Duration::from_millis(5);

/// The most game time one frame can cover. A hidden tab makes long frames (the browser stops
/// drawing it, and the next frame covers the whole gap), and the game shouldn't jump ahead.
const MAX_FRAME: Duration = Duration::from_millis(250);

#[wasm_bindgen]
impl Game {
    /// Call from a click or key press: the audio can only start after one.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Game {
        console_error_panic_hook::set_once();
        // Fails if called twice, which is harmless.
        let _ = console_log::init_with_level(log::Level::Info);

        let world = game::World::new(&launch_args(), audio::AudioSystem::new());
        let size = world.width() as u32;
        Game {
            world,
            frame: vec![0; (size * size * 4) as usize],
            frame_width: size,
            frame_height: size,
            game_time: Duration::ZERO,
        }
    }

    /// Advance the game and draw a frame, as the native `RedrawRequested` does, but advancing
    /// the game in equal steps of at most MAX_STEP.
    pub fn frame(&mut self, since_last_frame_ms: f64) {
        let since_last_frame =
            Duration::from_secs_f64(since_last_frame_ms.max(0.) / 1000.).min(MAX_FRAME);
        let num_steps = since_last_frame.div_duration_f64(MAX_STEP).ceil().max(1.) as u32;
        let step = since_last_frame / num_steps;
        for _ in 0..num_steps {
            self.game_time += step;
            // Only the first step can change the resolution: a key press is seen once, then
            // cleared.
            match self.world.update(step, self.game_time) {
                game::ResolutionChangeAction::DoNothing => {}
                game::ResolutionChangeAction::ChangeTo { x, y } => {
                    self.frame = vec![0; (x * y * 4) as usize];
                    self.frame_width = x;
                    self.frame_height = y;
                }
            }
        }
        self.world.draw(&mut self.frame);
    }

    pub fn frame_width(&self) -> u32 {
        self.frame_width
    }

    pub fn frame_height(&self) -> u32 {
        self.frame_height
    }

    /// Where the frame starts in the wasm memory.
    pub fn frame_ptr(&self) -> *const u8 {
        self.frame.as_ptr()
    }

    pub fn key_down(&mut self, code: &str) {
        if let Some(key) = key_code(code)
            && self.world.keys.insert(key)
        {
            self.world.first_pressed_this_frame.insert(key);
        }
    }

    pub fn key_up(&mut self, code: &str) {
        if let Some(key) = key_code(code) {
            self.world.keys.remove(&key);
        }
    }

    pub fn mouse_move(&mut self, dx: f64, dy: f64) {
        self.world.camera_mouse(dx, dy);
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}
