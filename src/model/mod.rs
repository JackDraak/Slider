//! Model layer containing core game logic and data structures.
//!
//! This module implements the puzzle mechanics, move validation, entropy calculations,
//! and puzzle state management. All components are independent of UI concerns.

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
