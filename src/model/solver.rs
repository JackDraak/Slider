use super::entropy::{EntropyCalculator, ShortestPathHeuristic};
use super::move_validator::{MoveValidator, Position};
use super::puzzle_state::PuzzleState;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Represents a state in the A* search
#[derive(Clone)]
struct SearchNode {
    state: PuzzleState,
    g_score: u32, // Cost from start (moves taken)
    h_score: u32, // Heuristic estimate to goal
    parent_index: Option<usize>, // Index into node storage vector
    move_from_parent: Option<Position>, // The move that led to this state
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

/// A* solver that finds the optimal solution path
pub struct AStarSolver {
    heuristic: ShortestPathHeuristic,
    max_iterations: usize,
}

impl AStarSolver {
    pub fn new() -> Self {
        Self {
            heuristic: ShortestPathHeuristic,
            max_iterations: 2_000_000, // Very generous limit - should handle all 4×4 puzzles
        }
    }

    /// Returns the length of the optimal solution, or None if unsolvable/timeout
    pub fn solve(&self, initial_state: &PuzzleState) -> Option<u32> {
        self.solve_with_path(initial_state).map(|path| path.len() as u32)
    }

    /// Returns the optimal solution path as a sequence of tile positions to move
    /// Returns None if unsolvable or timeout
    pub fn solve_with_path(&self, initial_state: &PuzzleState) -> Option<Vec<Position>> {
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
                let mut next_state = node_storage[current_idx].state.clone();
                if !next_state.apply_immediate_move(next_pos) {
                    continue;
                }

                let tentative_g = node_storage[current_idx].g_score + 1;
                let next_hash = self.state_hash(&next_state);

                // Skip if this state is already in closed set (fully explored)
                if closed_set.contains(&next_hash) {
                    continue;
                }

                // Skip if we've found a better path to this state
                if let Some(&best_g) = best_g_scores.get(&next_hash) {
                    if tentative_g >= best_g {
                        continue;
                    }
                }

                best_g_scores.insert(next_hash, tentative_g);

                let h_score = self.heuristic.calculate(&next_state);
                let next_node = SearchNode {
                    state: next_state,
                    g_score: tentative_g,
                    h_score,
                    parent_index: Some(current_idx),
                    move_from_parent: Some(next_pos),
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
        }

        None // No solution found
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

/// Entropy calculator that uses A* to compute the actual solution length
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
        self.solver.solve(state).unwrap_or(999) // Return high value if unsolvable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solver_solved_puzzle() {
        let puzzle = PuzzleState::new(3).unwrap();
        let solver = AStarSolver::new();

        let solution = solver.solve(&puzzle);
        assert_eq!(solution, Some(0));
    }

    #[test]
    fn test_solver_one_move() {
        let mut puzzle = PuzzleState::new(3).unwrap();
        puzzle.apply_immediate_move((2, 1));

        let solver = AStarSolver::new();
        let solution = solver.solve(&puzzle);

        assert_eq!(solution, Some(1));
    }

    #[test]
    fn test_solver_two_moves() {
        let mut puzzle = PuzzleState::new(3).unwrap();
        puzzle.apply_immediate_move((2, 1));
        puzzle.apply_immediate_move((2, 0));

        let solver = AStarSolver::new();
        let solution = solver.solve(&puzzle);

        assert_eq!(solution, Some(2));
    }

    #[test]
    fn test_solver_complex_puzzle() {
        let mut puzzle = PuzzleState::new(3).unwrap();
        // Make several moves
        puzzle.apply_immediate_move((2, 1));
        puzzle.apply_immediate_move((1, 1));
        puzzle.apply_immediate_move((0, 1));
        puzzle.apply_immediate_move((0, 2));

        let solver = AStarSolver::new();
        let solution = solver.solve(&puzzle);

        assert!(solution.is_some());
        assert!(solution.unwrap() <= 10); // Should find a solution within 10 moves
    }

    #[test]
    fn test_actual_solution_length_calculator() {
        let mut puzzle = PuzzleState::new(3).unwrap();
        puzzle.apply_immediate_move((2, 1));

        let calculator = ActualSolutionLength::new();
        let entropy = calculator.calculate(&puzzle);

        assert_eq!(entropy, 1);
    }

    #[test]
    fn test_solver_finds_optimal_path() {
        // Create a specific pattern where optimal path is known
        let mut puzzle = PuzzleState::new(3).unwrap();
        puzzle.apply_immediate_move((2, 1)); // Empty moves from (2,2) to (2,1)
        puzzle.apply_immediate_move((1, 1)); // Empty moves from (2,1) to (1,1)
        puzzle.apply_immediate_move((0, 1)); // Empty moves from (1,1) to (0,1)

        let solver = AStarSolver::new();
        let solution = solver.solve(&puzzle);

        // Optimal solution should be 3 moves (reverse the sequence)
        assert_eq!(solution, Some(3));
    }

    #[test]
    fn test_state_hash_uniqueness() {
        let puzzle1 = PuzzleState::new(3).unwrap();
        let mut puzzle2 = PuzzleState::new(3).unwrap();
        puzzle2.apply_immediate_move((2, 1));

        let solver = AStarSolver::new();
        let hash1 = solver.state_hash(&puzzle1);
        let hash2 = solver.state_hash(&puzzle2);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_state_hash_consistency() {
        let puzzle1 = PuzzleState::new(3).unwrap();
        let puzzle2 = PuzzleState::new(3).unwrap();

        let solver = AStarSolver::new();
        let hash1 = solver.state_hash(&puzzle1);
        let hash2 = solver.state_hash(&puzzle2);

        assert_eq!(hash1, hash2);
    }
}