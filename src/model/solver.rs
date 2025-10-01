use super::entropy::{EntropyCalculator, ShortestPathHeuristic};
use super::move_validator::{MoveValidator, Position};
use super::pattern_catalog::PatternCatalog;
use super::pattern_hash::PatternHashTable;
use super::puzzle_state::PuzzleState;
use super::relative_pattern::RelativePatternCatalog;
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
    moves_from_parent: Vec<Position>, // Move sequence that led to this state (single or pattern)
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
    use_patterns: bool, // Enable/disable pattern-based optimization
    pattern_catalog: Option<PatternCatalog>,  // Old absolute-position patterns
    relative_patterns: Option<RelativePatternCatalog>,  // Tile-agnostic patterns (slow)
    pattern_hash: Option<PatternHashTable>,  // Fast hash-table based patterns
}

impl AStarSolver {
    pub fn new() -> Self {
        Self {
            heuristic: ShortestPathHeuristic,
            max_iterations: 500_000,  // Reduced from 2M for better UI responsiveness
            use_patterns: false,
            pattern_catalog: None,
            relative_patterns: None,
            pattern_hash: None,
        }
    }

    /// Creates a solver with old absolute-position patterns (experimental)
    pub fn with_patterns(grid_size: usize) -> Self {
        Self {
            heuristic: ShortestPathHeuristic,
            max_iterations: 500_000,  // Reduced from 2M for better UI responsiveness
            use_patterns: true,
            pattern_catalog: Some(PatternCatalog::new(grid_size)),
            relative_patterns: None,
            pattern_hash: None,
        }
    }

    /// Creates a solver with tile-agnostic relative patterns (slow iteration)
    pub fn with_relative_patterns() -> Self {
        Self {
            heuristic: ShortestPathHeuristic,
            max_iterations: 500_000,  // Reduced from 2M for better UI responsiveness
            use_patterns: true,
            pattern_catalog: None,
            relative_patterns: Some(RelativePatternCatalog::new()),
            pattern_hash: None,
        }
    }

    /// Creates a solver with FAST hash-table based tile-agnostic patterns
    pub fn with_pattern_hash() -> Self {
        let catalog = RelativePatternCatalog::new();
        let patterns = catalog.patterns().to_vec();
        let hash_table = PatternHashTable::from_patterns(patterns);

        Self {
            heuristic: ShortestPathHeuristic,
            max_iterations: 500_000,  // Reduced from 2M for better UI responsiveness
            use_patterns: true,
            pattern_catalog: None,
            relative_patterns: None,
            pattern_hash: Some(hash_table),
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
            moves_from_parent: Vec::new(),
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
                self.explore_successor(
                    current_idx,
                    next_pos,
                    1, // Single move cost
                    &mut node_storage,
                    &mut open_set,
                    &mut closed_set,
                    &mut best_g_scores,
                );
            }

            // FAST hash-table based pattern exploration
            if self.use_patterns {
                if let Some(ref hash_table) = self.pattern_hash {
                    // O(1) lookup instead of O(patterns × 8)!
                    for pattern_match in hash_table.match_at(&node_storage[current_idx].state, empty_pos) {
                        let mut pattern_state = node_storage[current_idx].state.clone();
                        let mut pattern_valid = true;

                        for &move_pos in &pattern_match.moves {
                            if !pattern_state.apply_immediate_move(move_pos) {
                                pattern_valid = false;
                                break;
                            }
                        }

                        if pattern_valid {
                            let pattern_hash = self.state_hash(&pattern_state);

                            if closed_set.contains(&pattern_hash) {
                                continue;
                            }

                            let tentative_g = node_storage[current_idx].g_score + pattern_match.cost;

                            if let Some(&best_g) = best_g_scores.get(&pattern_hash) {
                                if tentative_g >= best_g {
                                    continue;
                                }
                            }

                            best_g_scores.insert(pattern_hash, tentative_g);

                            let h_score = self.heuristic.calculate(&pattern_state);
                            let pattern_node = SearchNode {
                                state: pattern_state,
                                g_score: tentative_g,
                                h_score,
                                parent_index: Some(current_idx),
                                moves_from_parent: pattern_match.moves,
                            };

                            let f_score = pattern_node.f_score();
                            let g_score = pattern_node.g_score;
                            let next_idx = node_storage.len();
                            node_storage.push(pattern_node);
                            open_set.push(HeapEntry {
                                f_score,
                                g_score,
                                node_index: next_idx,
                            });
                        }
                    }
                }

                // Tile-agnostic relative pattern exploration (slow - for comparison)
                if let Some(ref relative_catalog) = self.relative_patterns {
                    for pattern in relative_catalog.patterns() {
                        // Try to match pattern at current empty position
                        if let Some(moves) = pattern.match_at(&node_storage[current_idx].state, empty_pos) {
                            // Pattern matches! Apply the moves
                            let mut pattern_state = node_storage[current_idx].state.clone();
                            let mut pattern_valid = true;

                            for &move_pos in &moves {
                                if !pattern_state.apply_immediate_move(move_pos) {
                                    pattern_valid = false;
                                    break;
                                }
                            }

                            if pattern_valid {
                                let pattern_hash = self.state_hash(&pattern_state);

                                if closed_set.contains(&pattern_hash) {
                                    continue;
                                }

                                let tentative_g = node_storage[current_idx].g_score + pattern.cost;

                                if let Some(&best_g) = best_g_scores.get(&pattern_hash) {
                                    if tentative_g >= best_g {
                                        continue;
                                    }
                                }

                                best_g_scores.insert(pattern_hash, tentative_g);

                                let h_score = self.heuristic.calculate(&pattern_state);
                                let pattern_node = SearchNode {
                                    state: pattern_state,
                                    g_score: tentative_g,
                                    h_score,
                                    parent_index: Some(current_idx),
                                    moves_from_parent: moves,
                                };

                                let f_score = pattern_node.f_score();
                                let g_score = pattern_node.g_score;
                                let next_idx = node_storage.len();
                                node_storage.push(pattern_node);
                                open_set.push(HeapEntry {
                                    f_score,
                                    g_score,
                                    node_index: next_idx,
                                });
                            }
                        }
                    }
                }

                // Old absolute-position pattern exploration (for comparison)
                if let Some(ref catalog) = self.pattern_catalog {
                    for pattern in catalog.patterns() {
                        // Try applying the entire pattern sequence
                        let mut pattern_state = node_storage[current_idx].state.clone();
                        let mut pattern_valid = true;

                        // Apply all moves in the pattern
                        for &move_pos in &pattern.moves {
                            if !pattern_state.apply_immediate_move(move_pos) {
                                pattern_valid = false;
                                break;
                            }
                        }

                        if pattern_valid {
                            // Pattern successfully applied - add resulting state to open set
                            let pattern_hash = self.state_hash(&pattern_state);

                            // Skip if already explored
                            if closed_set.contains(&pattern_hash) {
                                continue;
                            }

                            let tentative_g = node_storage[current_idx].g_score + pattern.cost;

                            // Skip if we've found a better path to this state
                            if let Some(&best_g) = best_g_scores.get(&pattern_hash) {
                                if tentative_g >= best_g {
                                    continue;
                                }
                            }

                            best_g_scores.insert(pattern_hash, tentative_g);

                            let h_score = self.heuristic.calculate(&pattern_state);
                            let pattern_node = SearchNode {
                                state: pattern_state,
                                g_score: tentative_g,
                                h_score,
                                parent_index: Some(current_idx),
                                // Store entire pattern sequence for path reconstruction
                                moves_from_parent: pattern.moves.clone(),
                            };

                            let f_score = pattern_node.f_score();
                            let g_score = pattern_node.g_score;
                            let next_idx = node_storage.len();
                            node_storage.push(pattern_node);
                            open_set.push(HeapEntry {
                                f_score,
                                g_score,
                                node_index: next_idx,
                            });
                        }
                    }
                }
            }
        }

        None // No solution found
    }

    /// Helper to explore a successor state (extracted for code reuse)
    fn explore_successor(
        &self,
        current_idx: usize,
        move_pos: Position,
        move_cost: u32,
        node_storage: &mut Vec<SearchNode>,
        open_set: &mut BinaryHeap<HeapEntry>,
        closed_set: &HashSet<u64>,
        best_g_scores: &mut HashMap<u64, u32>,
    ) {
        let mut next_state = node_storage[current_idx].state.clone();
        if !next_state.apply_immediate_move(move_pos) {
            return;
        }

        let tentative_g = node_storage[current_idx].g_score + move_cost;
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
            moves_from_parent: vec![move_pos],
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

        // Walk backwards from goal to start, collecting move sequences
        while let Some(parent_idx) = node_storage[current_idx].parent_index {
            // Add all moves from this edge (could be single move or pattern sequence)
            for &move_pos in node_storage[current_idx].moves_from_parent.iter().rev() {
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
    fn test_pattern_solver_finds_optimal_solution() {
        // Create a simple puzzle
        let mut puzzle = PuzzleState::new(3).unwrap();
        puzzle.apply_immediate_move((2, 1));
        puzzle.apply_immediate_move((1, 1));
        puzzle.apply_immediate_move((0, 1));

        // Solve with both solvers
        let solver_normal = AStarSolver::new();
        let solver_patterns = AStarSolver::with_patterns(3);

        let solution_normal = solver_normal.solve(&puzzle);
        let solution_patterns = solver_patterns.solve(&puzzle);

        // Both should find the same optimal solution length
        assert_eq!(solution_normal, solution_patterns);
        assert_eq!(solution_normal, Some(3));
    }

    #[test]
    fn test_pattern_solver_path_correctness() {
        // Create a simple puzzle and solve with patterns
        let mut puzzle = PuzzleState::new(3).unwrap();
        puzzle.apply_immediate_move((2, 1));

        let solver_patterns = AStarSolver::with_patterns(3);
        let path = solver_patterns.solve_with_path(&puzzle).unwrap();

        // Verify the path actually solves the puzzle
        let mut test_state = puzzle.clone();
        for move_pos in path {
            assert!(test_state.apply_immediate_move(move_pos),
                "Path contains invalid move: {:?}", move_pos);
        }
        assert!(test_state.is_solved(), "Path does not lead to solved state");
    }

    #[test]
    fn test_relative_pattern_solver_finds_optimal_solution() {
        // Create a simple puzzle
        let mut puzzle = PuzzleState::new(3).unwrap();
        puzzle.apply_immediate_move((2, 1));
        puzzle.apply_immediate_move((1, 1));
        puzzle.apply_immediate_move((0, 1));

        // Solve with both normal and relative pattern solvers
        let solver_normal = AStarSolver::new();
        let solver_relative = AStarSolver::with_relative_patterns();

        let solution_normal = solver_normal.solve(&puzzle);
        let solution_relative = solver_relative.solve(&puzzle);

        // Both should find the same optimal solution length
        assert_eq!(solution_normal, solution_relative);
        assert_eq!(solution_normal, Some(3));
    }

    #[test]
    fn test_relative_pattern_solver_path_correctness() {
        // Create a simple puzzle and solve with relative patterns
        let mut puzzle = PuzzleState::new(4).unwrap();
        puzzle.apply_immediate_move((3, 2));
        puzzle.apply_immediate_move((2, 2));

        let solver_relative = AStarSolver::with_relative_patterns();
        let path = solver_relative.solve_with_path(&puzzle).unwrap();

        // Verify the path actually solves the puzzle
        let mut test_state = puzzle.clone();
        for move_pos in path {
            assert!(test_state.apply_immediate_move(move_pos),
                "Path contains invalid move: {:?}", move_pos);
        }
        assert!(test_state.is_solved(), "Path does not lead to solved state");
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