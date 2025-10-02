//! # Model Layer - Core Game Logic
//!
//! This module contains the core game logic and data structures for the sliding tile puzzle.
//! All components are completely independent of UI concerns and can be used programmatically.
//!
//! ## Core Components
//!
//! - [`PuzzleState`] - The main game state representing the tile grid
//! - [`MoveValidator`] - Validates legal moves and resolves chain moves
//! - [`Tile`] - Individual tile representation with content abstraction
//! - [`Position`] - Grid position type with validation
//!
//! ## Entropy Calculation
//!
//! Multiple entropy calculators provide different measures of puzzle complexity:
//!
//! - [`ManhattanDistance`] - Fast heuristic based on tile displacement
//! - [`ShortestPathHeuristic`] - Enhanced heuristic with linear conflict detection
//! - [`ActualSolutionLength`] - Exact optimal solution length using A* search
//! - [`EnhancedHeuristic`] - Combined heuristic for improved accuracy
//!
//! ## Solving Algorithms
//!
//! - [`AStarSolver`] - Optimal pathfinding using the A* algorithm
//! - Supports cancellation and configurable iteration limits
//! - Memory-efficient implementation using indexed storage
//!
//! ## Difficulty Levels
//!
//! Predefined difficulty thresholds based on entropy measurements:
//! - [`Difficulty::Easy`] - Low entropy puzzles
//! - [`Difficulty::Medium`] - Moderate complexity
//! - [`Difficulty::Hard`] - High complexity requiring more moves
//!
//! ## Performance Monitoring
//!
//! - [`PerformanceMetrics`] - Tracks algorithm execution times
//! - [`PerformanceTimer`] - High-precision timing utilities
//!
//! ## Error Handling
//!
//! Comprehensive error types for different failure modes:
//! - [`PuzzleError`] - General puzzle-related errors
//! - [`SolverError`] - A* solver specific errors
//! - [`AutoSolveError`] - Auto-solve operation errors
//!
//! ## Example Usage
//!
//! ```rust
//! use slider::model::{PuzzleState, Difficulty, ManhattanDistance};
//!
//! // Create a new 4×4 puzzle
//! let mut puzzle = PuzzleState::new(4)?;
//!
//! // Calculate entropy
//! let entropy = ManhattanDistance.calculate(&puzzle);
//! println!("Initial entropy: {}", entropy);
//!
//! // Make a move
//! puzzle.apply_immediate_move((3, 2))?;
//!
//! // Check if solved
//! if puzzle.is_solved() {
//!     println!("Puzzle is solved!");
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod enhanced_heuristic;
pub mod entropy;
pub mod error;
pub mod move_validator;
pub mod performance;
pub mod puzzle_state;
pub mod solver;
pub mod tile;

pub use enhanced_heuristic::EnhancedHeuristic;
pub use entropy::{Difficulty, EntropyCalculator, ManhattanDistance, ShortestPathHeuristic};
pub use error::{AutoSolveError, PuzzleError, SolverError};
pub use move_validator::{MoveValidator, Position};
pub use performance::{PerformanceMetrics, PerformanceTimer};
pub use puzzle_state::PuzzleState;
pub use solver::{AStarSolver, ActualSolutionLength};
pub use tile::{Tile, TileContent};
