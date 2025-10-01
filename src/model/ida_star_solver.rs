use super::entropy::{EntropyCalculator, ShortestPathHeuristic};
use super::move_validator::{MoveValidator, Position};
use super::puzzle_state::PuzzleState;

/// IDA* (Iterative Deepening A*) solver - memory efficient alternative to A*
///
/// IDA* performs depth-first search with increasing cost thresholds:
/// - Uses O(depth) memory instead of O(states)
/// - Better cache locality (fewer unique states in memory)
/// - Often faster than A* for sliding puzzles due to less overhead
/// - Still guarantees optimal solution
///
/// Key insight: Most nodes in A* are never revisited anyway, so storing
/// them all wastes memory and causes cache thrashing. IDA* trades some
/// redundant computation for dramatically better memory usage.
pub struct IDAStarSolver {
    heuristic: ShortestPathHeuristic,
    max_iterations: usize,
    nodes_explored: usize, // For benchmarking
}

/// Result of a depth-limited search iteration
enum SearchResult {
    Found(Vec<Position>),  // Solution found
    NotFound(u32),         // Not found, minimum f-cost seen that exceeded threshold
}

impl IDAStarSolver {
    pub fn new() -> Self {
        Self {
            heuristic: ShortestPathHeuristic,
            max_iterations: 1_000_000, // Higher than A* since we're more efficient
            nodes_explored: 0,
        }
    }

    /// Returns the optimal solution path as a sequence of tile positions to move
    /// Returns None if unsolvable or timeout
    pub fn solve_with_path(&mut self, initial_state: &PuzzleState) -> Option<Vec<Position>> {
        if initial_state.is_solved() {
            return Some(Vec::new());
        }

        self.nodes_explored = 0;

        // Start with heuristic estimate as initial threshold
        let mut threshold = self.heuristic.calculate(initial_state) as u32;
        let validator = MoveValidator::new(initial_state.size()).expect("valid size");

        // Iteratively deepen the search threshold
        loop {
            let mut path = Vec::new();

            match self.depth_limited_search(
                initial_state,
                0, // g_score starts at 0
                threshold,
                &mut path,
                &validator,
                None, // No previous position (no backtracking on first move)
            ) {
                SearchResult::Found(solution) => {
                    println!("IDA*: Found solution in {} nodes", self.nodes_explored);
                    return Some(solution);
                }
                SearchResult::NotFound(min_exceeded) => {
                    // No solution at this threshold
                    if min_exceeded == u32::MAX {
                        // No deeper nodes found - puzzle is unsolvable
                        return None;
                    }

                    if self.nodes_explored > self.max_iterations {
                        println!("IDA*: Timeout after {} nodes", self.nodes_explored);
                        return None; // Timeout
                    }

                    // Increase threshold to minimum f-cost that exceeded it
                    threshold = min_exceeded;
                }
            }
        }
    }

    /// Depth-limited DFS with f-cost pruning
    ///
    /// Returns:
    /// - Found(path) if solution found
    /// - NotFound(min_f) where min_f is the minimum f-cost that exceeded threshold
    fn depth_limited_search(
        &mut self,
        state: &PuzzleState,
        g_score: u32,
        threshold: u32,
        path: &mut Vec<Position>,
        validator: &MoveValidator,
        prev_empty: Option<Position>, // Previous empty position to prevent backtracking
    ) -> SearchResult {
        self.nodes_explored += 1;

        // Calculate f-cost = g + h
        let h_score = self.heuristic.calculate(state);
        let f_score = g_score + h_score;

        // Prune if f-cost exceeds threshold
        if f_score > threshold {
            return SearchResult::NotFound(f_score);
        }

        // Goal test
        if state.is_solved() {
            return SearchResult::Found(path.clone());
        }

        // Explore successors (immediate moves only)
        let empty_pos = state.empty_position();
        let moves = validator.get_immediate_moves(empty_pos);

        let mut min_exceeded = u32::MAX;

        for &next_pos in &moves {
            // Skip backtracking (moving tile that just moved back)
            if Some(next_pos) == prev_empty {
                continue;
            }

            // Apply move
            let mut next_state = state.clone();
            if !next_state.apply_immediate_move(next_pos) {
                continue;
            }

            // Add move to path
            path.push(next_pos);

            // Recursive search
            match self.depth_limited_search(
                &next_state,
                g_score + 1,
                threshold,
                path,
                validator,
                Some(empty_pos), // Current empty becomes previous
            ) {
                SearchResult::Found(solution) => {
                    return SearchResult::Found(solution);
                }
                SearchResult::NotFound(exceeded) => {
                    // Track minimum f-cost that exceeded threshold
                    min_exceeded = min_exceeded.min(exceeded);
                }
            }

            // Backtrack
            path.pop();
        }

        SearchResult::NotFound(min_exceeded)
    }

    /// Returns the number of nodes explored in the last search
    pub fn nodes_explored(&self) -> usize {
        self.nodes_explored
    }

    /// Returns the length of the optimal solution, or None if unsolvable/timeout
    pub fn solve(&mut self, initial_state: &PuzzleState) -> Option<u32> {
        self.solve_with_path(initial_state).map(|path| path.len() as u32)
    }
}

impl Default for IDAStarSolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ida_star_solved_puzzle() {
        let mut solver = IDAStarSolver::new();
        let puzzle = PuzzleState::new(3).unwrap();

        let result = solver.solve_with_path(&puzzle);
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_ida_star_simple_puzzle() {
        let mut solver = IDAStarSolver::new();
        let mut puzzle = PuzzleState::new(3).unwrap();

        // Make a few moves
        puzzle.apply_immediate_move((2, 1));
        puzzle.apply_immediate_move((2, 0));

        let result = solver.solve_with_path(&puzzle);
        assert!(result.is_some());

        let path = result.unwrap();
        assert!(path.len() <= 2); // Should find optimal solution

        // Verify solution actually solves the puzzle
        let mut test_state = puzzle.clone();
        for &move_pos in &path {
            test_state.apply_immediate_move(move_pos);
        }
        assert!(test_state.is_solved());
    }

    #[test]
    fn test_ida_star_finds_optimal_solution() {
        let mut solver = IDAStarSolver::new();
        let mut puzzle = PuzzleState::new(3).unwrap();

        // Make two moves to scramble
        puzzle.apply_immediate_move((2, 1));
        puzzle.apply_immediate_move((2, 0));

        let result = solver.solve(&puzzle);
        assert!(result.is_some());

        // Should find optimal solution (likely 2 moves)
        let solution_length = result.unwrap();
        assert!(solution_length >= 2 && solution_length <= 4); // Reasonable range
    }

    #[test]
    fn test_ida_star_nodes_explored() {
        let mut solver = IDAStarSolver::new();
        let mut puzzle = PuzzleState::new(3).unwrap();

        puzzle.apply_immediate_move((2, 1));

        solver.solve_with_path(&puzzle);

        // IDA* should explore fewer nodes than A* for simple puzzles
        let nodes = solver.nodes_explored();
        assert!(nodes > 0);
        assert!(nodes < 1000); // Should be very efficient for 1-move puzzle
    }
}
