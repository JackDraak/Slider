//! Controller layer orchestrating game logic and user interactions.
//!
//! This module contains the game controller (handling player moves and game state)
//! and the shuffle controller (generating solvable puzzles with entropy requirements).

pub mod game_controller;
pub mod shuffle_controller;

pub use game_controller::{EntropyMetrics, GameController, MoveHistory};
pub use shuffle_controller::ShuffleController;