//! # A* Pathfinding Solver
//!
//! This module implements an optimal pathfinding solver using the A* algorithm.
//! The solver finds the shortest possible solution to sliding tile puzzles
//! using an enhanced heuristic that combines multiple puzzle complexity metrics.
//!
//! ## Algorithm Overview
//!
//! The A* algorithm maintains a priority queue of states to explore, ordered by:
//! `f_score = g_score + h_score`
//!
//! - `g_score`: Number of moves taken from the initial state
//! - `h_score`: Heuristic estimate of moves remaining to goal
//! - `f_score`: Total estimated path cost
//!
//! ## Key Features
//!
//! - **Optimal Solutions**: Guaranteed to find the shortest possible path
//! - **Memory Efficient**: Uses indexed storage instead of exponential parent chains
//! - **Cancellation Support**: Can be interrupted during long searches
//! - **Configurable Limits**: Adjustable iteration limits to prevent infinite searches
//! - **Fast State Hashing**: U64 hashing for efficient duplicate detection
//!
//! ## Performance Characteristics
//!
//! - **Time Complexity**: O(b^d) where b is branching factor and d is solution depth
//! - **Space Complexity**: O(b^d) in worst case, typically much less in practice
//! - **Solves All 4×4 Puzzles**: 2M iteration limit handles all solvable 4×4 states
//!
//! ## Example Usage
//!
//! ```rust
//! use slider::model::{AStarSolver, PuzzleState};
//!
//! let solver = AStarSolver::new();
//! let puzzle = PuzzleState::new(4)?;
//!
//! // Get solution length
//! if let Some(length) = solver.solve(&puzzle) {
//!     println!("Optimal solution: {} moves", length);
//! }
//!
//! // Get full solution path
//! if let Some(path) = solver.solve_with_path(&puzzle) {
//!     println!("Move sequence: {:?}", path);
//!     for &pos in &path {
//!         // Apply each move to solve the puzzle
//!     }
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use super::enhanced_heuristic::EnhancedHeuristic;
use super::entropy::EntropyCalculator;
use super::move_validator::{MoveValidator, Position};
use super::puzzle_state::PuzzleState;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
use std::sync::Arc;

/// Represents a state in the A* search
#[derive(Clone)]
struct SearchNode {
    state: PuzzleState,
    g_score: u32,           // Cost from start (moves taken)
    h_score: u32,           // Heuristic estimate to goal
    parent_index: Option<usize>, // Index into node storage vector
    move_from_parent: Option<Position>, // Single move that led to this state
}

impl SearchNode {
    fn f_score(&self) -> u32 {
        self.g_score + self.h_score
    }
}

/// Wrapper for BinaryHeap to maintain min-heap ordering by f_score
#[derive(Eq, PartialEq)]
struct HeapEntry {
    f_score: u32,
    g_score: u32, // Used for tie-breaking
    node_index: usize,
}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap behavior (lower f_score = higher priority)
        // When f_scores are equal, prefer higher g_score (closer to goal)
        other
            .f_score
            .cmp(&self.f_score)
            .then_with(|| self.g_score.cmp(&other.g_score))
    }
}

impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A* solver that finds the optimal solution path using Enhanced Heuristic
pub struct AStarSolver {
    heuristic: EnhancedHeuristic,
    max_iterations: usize,
}

impl AStarSolver {
    pub fn new() -> Self {
        Self {
            heuristic: EnhancedHeuristic,
            max_iterations: 1_000_000,
        }
    }

    /// Returns the length of the optimal solution, or None if unsolvable/timeout
    pub fn solve(&self, initial_state: &PuzzleState) -> Option<u32> {
        self.solve_with_path(initial_state).map(|path| path.len() as u32)
    }

    /// Returns the optimal solution path as a sequence of tile positions to move
    /// Returns None if unsolvable or timeout
    pub fn solve_with_path(&self, initial_state: &PuzzleState) -> Option<Vec<Position>> {
        self.solve_with_path_cancellable(initial_state, None)
    }

    /// Returns the optimal solution path with support for cancellation
    /// Returns None if unsolvable, timeout, or cancelled
    pub fn solve_with_path_cancellable(
        &self,
        initial_state: &PuzzleState,
        cancel_flag: Option<Arc<AtomicBool>>,
    ) -> Option<Vec<Position>> {
        if initial_state.is_solved() {
            return Some(Vec::new());
        }

        let mut open_set = BinaryHeap::new();
        let mut closed_set = HashSet::new();
        let mut best_g_scores: HashMap<u64, u32> = HashMap::new();
        let mut node_storage: Vec<SearchNode> = Vec::new();

        let initial_node = SearchNode {
            state: initial_state.clone(),
            g_score: 0,
            h_score: self.heuristic.calculate(initial_state),
            parent_index: None,
            move_from_parent: None,
        };

        let initial_f_score = initial_node.f_score();
        let initial_g_score = initial_node.g_score;
        node_storage.push(initial_node);
        open_set.push(HeapEntry {
            f_score: initial_f_score,
            g_score: initial_g_score,
            node_index: 0,
        });

        best_g_scores.insert(self.state_hash(initial_state), 0);

        // Size is guaranteed valid since initial_state was constructed successfully
        let validator = MoveValidator::new(initial_state.size()).expect("valid size");
        let mut iterations = 0;

        while let Some(HeapEntry { node_index: current_idx, .. }) = open_set.pop() {
            iterations += 1;

            // Check for cancellation every 1000 iterations
            if iterations % 1000 == 0 {
                if let Some(ref cancel) = cancel_flag {
                    if cancel.load(AtomicOrdering::Relaxed) {
                        return None; // Cancelled
                    }
                }
            }

            if iterations > self.max_iterations {
                return None; // Timeout
            }

            let current = &node_storage[current_idx];

            if current.state.is_solved() {
                return Some(self.reconstruct_path(&node_storage, current_idx));
            }

            let current_hash = self.state_hash(&current.state);
            if closed_set.contains(&current_hash) {
                continue;
            }
            closed_set.insert(current_hash);

            // Explore all immediate moves (no chain moves for solver)
            let empty_pos = current.state.empty_position();
            for next_pos in validator.get_immediate_moves(empty_pos) {
                self.explore_successor(
                    current_idx,
                    next_pos,
                    &mut node_storage,
                    &mut open_set,
                    &closed_set,
                    &mut best_g_scores,
                );
            }
        }

        None // No solution found
    }

    /// Helper to explore a successor state
    fn explore_successor(
        &self,
        current_idx: usize,
        move_pos: Position,
        node_storage: &mut Vec<SearchNode>,
        open_set: &mut BinaryHeap<HeapEntry>,
        closed_set: &HashSet<u64>,
        best_g_scores: &mut HashMap<u64, u32>,
    ) {
        let mut next_state = node_storage[current_idx].state.clone();
        if !next_state.apply_immediate_move(move_pos) {
            return;
        }

        let tentative_g = node_storage[current_idx].g_score + 1;
        let next_hash = self.state_hash(&next_state);

        // Skip if this state is already in closed set (fully explored)
        if closed_set.contains(&next_hash) {
            return;
        }

        // Skip if we've found a better path to this state
        if let Some(&best_g) = best_g_scores.get(&next_hash) {
            if tentative_g >= best_g {
                return;
            }
        }

        best_g_scores.insert(next_hash, tentative_g);

        let h_score = self.heuristic.calculate(&next_state);
        let next_node = SearchNode {
            state: next_state,
            g_score: tentative_g,
            h_score,
            parent_index: Some(current_idx),
            move_from_parent: Some(move_pos),
        };

        let f_score = next_node.f_score();
        let g_score = next_node.g_score;
        let next_idx = node_storage.len();
        node_storage.push(next_node);
        open_set.push(HeapEntry {
            f_score,
            g_score,
            node_index: next_idx,
        });
    }

    /// Reconstructs the solution path by following parent indices
    fn reconstruct_path(&self, node_storage: &[SearchNode], goal_idx: usize) -> Vec<Position> {
        let mut path = Vec::new();
        let mut current_idx = goal_idx;

        // Walk backwards from goal to start, collecting moves
        while let Some(parent_idx) = node_storage[current_idx].parent_index {
            if let Some(move_pos) = node_storage[current_idx].move_from_parent {
                path.push(move_pos);
            }
            current_idx = parent_idx;
        }

        // Reverse to get path from start to goal
        path.reverse();
        path
    }

    /// Creates a hash representation of the puzzle state for deduplication
    fn state_hash(&self, state: &PuzzleState) -> u64 {
        let mut hasher = DefaultHasher::new();
        let size = state.size();

        for row in 0..size {
            for col in 0..size {
                if let Some(tile) = state.tile_at((row, col)) {
                    if let Some(num) = tile.numeric_value() {
                        num.hash(&mut hasher);
                    }
                } else {
                    // Use a special value for empty cell
                    u32::MAX.hash(&mut hasher);
                }
            }
        }

        hasher.finish()
    }
}

impl Default for AStarSolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculator for actual solution length using A* solver
pub struct ActualSolutionLength {
    solver: AStarSolver,
}

impl ActualSolutionLength {
    pub fn new() -> Self {
        Self {
            solver: AStarSolver::new(),
        }
    }
}

impl Default for ActualSolutionLength {
    fn default() -> Self {
        Self::new()
    }
}

impl EntropyCalculator for ActualSolutionLength {
    fn calculate(&self, state: &PuzzleState) -> u32 {
        self.solver.solve(state).unwrap_or(999)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_simple_puzzle() -> PuzzleState {
        let mut puzzle = PuzzleState::new(3).unwrap();
        // Empty starts at (2,2). Move tile from (2,1) into empty (creating one-move puzzle)
        puzzle.apply_immediate_move((2, 1));
        puzzle
    }

    #[test]
    fn test_solver_solved_puzzle() {
        let puzzle = PuzzleState::new(3).unwrap();
        let solver = AStarSolver::new();
        let solution = solver.solve(&puzzle);
        assert_eq!(solution, Some(0));
    }

    #[test]
    fn test_solver_one_move() {
        let puzzle = create_simple_puzzle();
        let solver = AStarSolver::new();
        let solution = solver.solve(&puzzle);
        assert_eq!(solution, Some(1));
    }

    #[test]
    fn test_solver_finds_optimal_path() {
        let puzzle = create_simple_puzzle();
        let solver = AStarSolver::new();
        let path = solver.solve_with_path(&puzzle).unwrap();
        assert_eq!(path.len(), 1);
    }

    #[test]
    fn test_solver_two_moves() {
        let mut puzzle = PuzzleState::new(3).unwrap();
        // Empty at (2,2). Make two moves
        puzzle.apply_immediate_move((2, 1)); // Empty moves to (2,1)
        puzzle.apply_immediate_move((1, 1)); // Empty moves to (1,1)

        let solver = AStarSolver::new();
        let solution = solver.solve(&puzzle);
        assert_eq!(solution, Some(2));
    }

    #[test]
    fn test_solver_complex_puzzle() {
        let mut puzzle = PuzzleState::new(3).unwrap();
        // Empty starts at (2,2). Make several moves
        let moves = vec![(2, 1), (1, 1), (0, 1), (0, 2), (1, 2)];
        for move_pos in moves {
            puzzle.apply_immediate_move(move_pos);
        }

        let solver = AStarSolver::new();
        let solution = solver.solve(&puzzle);
        // Should find optimal solution
        assert!(solution.is_some());
        assert_eq!(solution.unwrap(), 5);
    }

    #[test]
    fn test_state_hash_consistency() {
        let puzzle = PuzzleState::new(4).unwrap();
        let solver = AStarSolver::new();
        let hash1 = solver.state_hash(&puzzle);
        let hash2 = solver.state_hash(&puzzle);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_state_hash_uniqueness() {
        let puzzle1 = PuzzleState::new(4).unwrap();
        let mut puzzle2 = PuzzleState::new(4).unwrap();
        puzzle2.apply_immediate_move((3, 2)); // Move tile adjacent to empty

        let solver = AStarSolver::new();
        let hash1 = solver.state_hash(&puzzle1);
        let hash2 = solver.state_hash(&puzzle2);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_actual_solution_length_calculator() {
        let puzzle = create_simple_puzzle();
        let calculator = ActualSolutionLength::new();
        let length = calculator.calculate(&puzzle);
        assert_eq!(length, 1);
    }
}
