//! # Slider - Sliding Tile Puzzle Game
//!
//! A Rust implementation of the classic sliding-tile puzzle (also known as the "fifteen puzzle").
//! This implementation features entropy-based difficulty levels, guaranteed solvable shuffles,
//! and a clean Model-Controller-Presenter architecture.
//!
//! ## Architecture
//!
//! The codebase follows the **Model-Controller-Presenter (MCP)** pattern:
//!
//! - **Model** ([`model`]): Core game logic, puzzle state, move validation, and entropy calculations
//! - **Controller** ([`controller`]): Game orchestration, shuffling, and move history
//! - **Presenter** ([`presenter`]): GUI rendering using egui framework
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use slider::run_gui;
//!
//! fn main() -> eframe::Result<()> {
//!     // Run with default 4×4 grid
//!     run_gui(4)
//! }
//! ```
//!
//! ## Features
//!
//! - Grid sizes from 3×3 to 22×22
//! - Immediate moves and chain moves (click any tile in line with empty cell)
//! - **Smooth tile animations** - 200ms ease-out cubic interpolation
//! - **Auto-solve with A\* pathfinding**:
//!   - Watch optimal solutions play out with smooth animations
//!   - Recalculates fresh path on each invocation
//!   - Never times out on 4×4 puzzles (2M iteration limit)
//! - Three entropy calculation algorithms:
//!   - Manhattan Distance (fast heuristic, microseconds)
//!   - Shortest Path with linear conflicts (improved heuristic)
//!   - A\* Actual Solution Length (exact optimal path, milliseconds)
//! - Three difficulty levels with entropy-based thresholds
//! - Guaranteed solvable shuffles using mechanical simulation
//! - Performance metrics with calculation timing for A\* solver
//! - Move counter and real-time entropy display
//! - Visual feedback with color-coded tiles and smooth animations
//! - Proper error handling (Result types, no panics in library code)
//! - Debug logging for auto-solve behavior verification
//!
//! ## Example: Programmatic Usage
//!
//! ```rust
//! use slider::{GameController, Difficulty};
//!
//! // Create a new 4×4 puzzle
//! let mut game = GameController::new(4).unwrap();
//!
//! // Start a new game with medium difficulty
//! game.new_game(Difficulty::Medium);
//!
//! // Make a move by clicking a tile position
//! let success = game.handle_click((3, 2));
//!
//! // Check if solved
//! if game.is_solved() {
//!     println!("Puzzle solved in {} moves!", game.move_count());
//! }
//! ```

pub mod controller;
pub mod model;
pub mod presenter;

pub use controller::{GameController, ShuffleController};
pub use model::{Difficulty, PuzzleState};
pub use presenter::run_gui;