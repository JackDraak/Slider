//! Model layer containing core game logic and data structures.
//!
//! This module implements the puzzle mechanics, move validation, entropy calculations,
//! and puzzle state management. All components are independent of UI concerns.

pub mod entropy;
pub mod empty_cell_path;
pub mod error;
pub mod ida_star_solver;
pub mod move_validator;
pub mod pattern_catalog;
pub mod pattern_hash;
pub mod performance;
pub mod puzzle_state;
pub mod relative_pattern;
pub mod solver;
pub mod solver_benchmark;
pub mod tile;
pub mod walking_distance;

pub use entropy::{Difficulty, EntropyCalculator, ManhattanDistance, ShortestPathHeuristic};
pub use empty_cell_path::EmptyCellPathHeuristic;
pub use error::{AutoSolveError, PuzzleError, SolverError};
pub use move_validator::{MoveValidator, Position};
pub use pattern_catalog::{MovePattern, PatternCatalog, PatternType};
pub use pattern_hash::{PatternHashTable, PatternMatch};
pub use performance::{PerformanceMetrics, PerformanceTimer};
pub use puzzle_state::PuzzleState;
pub use relative_pattern::{RelativeMove, RelativePattern, RelativePatternCatalog, Transform};
pub use solver::{AStarSolver, AStarSolverEmptyCell, ActualSolutionLength};
pub use ida_star_solver::IDAStarSolver;
pub use solver_benchmark::{SolverBenchmarkResult, compare_solvers, run_comprehensive_benchmark};
pub use tile::{Tile, TileContent};
pub use walking_distance::WalkingDistance;
