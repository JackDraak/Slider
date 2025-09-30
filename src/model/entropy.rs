use super::puzzle_state::PuzzleState;

/// Trait for calculating entropy (disorder) in the puzzle
pub trait EntropyCalculator {
    /// Calculate the entropy value for the given puzzle state
    /// Higher values indicate more disorder
    fn calculate(&self, state: &PuzzleState) -> u32;
}

/// Calculates entropy using Manhattan Distance
/// Sum of distances from each tile's current position to its home position
#[derive(Debug, Default)]
pub struct ManhattanDistance;

impl EntropyCalculator for ManhattanDistance {
    fn calculate(&self, state: &PuzzleState) -> u32 {
        let mut total_distance = 0u32;

        for (current_pos, tile) in state.tiles() {
            let (current_row, current_col) = current_pos;
            let (home_row, home_col) = tile.home_position;

            let distance = current_row.abs_diff(home_row) + current_col.abs_diff(home_col);
            total_distance += distance as u32;
        }

        total_distance
    }
}

/// Calculates entropy using a heuristic for shortest path length
/// This is an approximation - actual shortest path requires A* search
/// Uses Manhattan distance as the heuristic (admissible and consistent)
#[derive(Debug, Default)]
pub struct ShortestPathHeuristic;

impl EntropyCalculator for ShortestPathHeuristic {
    fn calculate(&self, state: &PuzzleState) -> u32 {
        // For now, use Manhattan distance as the heuristic
        // This gives a lower bound on the actual number of moves needed
        let manhattan = ManhattanDistance;
        let base_score = manhattan.calculate(state);

        // Add linear conflict penalty: tiles in correct row/col but blocking each other
        let linear_conflicts = count_linear_conflicts(state);

        base_score + (linear_conflicts * 2)
    }
}

/// Counts linear conflicts: pairs of tiles in the same row or column
/// that are in their target row/column but in reverse order
fn count_linear_conflicts(state: &PuzzleState) -> u32 {
    let mut conflicts = 0;

    // Check row conflicts
    for row in 0..state.size() {
        let mut tiles_in_row = Vec::new();
        for col in 0..state.size() {
            if let Some(tile) = state.tile_at((row, col)) {
                let (home_row, home_col) = tile.home_position;
                if home_row == row {
                    tiles_in_row.push((col, home_col));
                }
            }
        }

        // Count inversions in this row
        for i in 0..tiles_in_row.len() {
            for j in (i + 1)..tiles_in_row.len() {
                let (curr_i, target_i) = tiles_in_row[i];
                let (curr_j, target_j) = tiles_in_row[j];
                if curr_i < curr_j && target_i > target_j {
                    conflicts += 1;
                }
            }
        }
    }

    // Check column conflicts
    for col in 0..state.size() {
        let mut tiles_in_col = Vec::new();
        for row in 0..state.size() {
            if let Some(tile) = state.tile_at((row, col)) {
                let (home_row, home_col) = tile.home_position;
                if home_col == col {
                    tiles_in_col.push((row, home_row));
                }
            }
        }

        // Count inversions in this column
        for i in 0..tiles_in_col.len() {
            for j in (i + 1)..tiles_in_col.len() {
                let (curr_i, target_i) = tiles_in_col[i];
                let (curr_j, target_j) = tiles_in_col[j];
                if curr_i < curr_j && target_i > target_j {
                    conflicts += 1;
                }
            }
        }
    }

    conflicts
}

/// Difficulty levels based on entropy thresholds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    /// Returns the minimum entropy threshold for this difficulty level
    /// These are heuristic values that may need tuning based on grid size
    pub fn min_entropy(&self, grid_size: usize) -> u32 {
        let scale = (grid_size * grid_size) as u32;
        match self {
            Difficulty::Easy => scale / 2,
            Difficulty::Medium => scale,
            Difficulty::Hard => scale * 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solved_puzzle_zero_entropy() {
        let puzzle = PuzzleState::new(4).unwrap();
        let calculator = ManhattanDistance;
        assert_eq!(calculator.calculate(&puzzle), 0);
    }

    #[test]
    fn test_manhattan_distance_single_move() {
        let mut puzzle = PuzzleState::new(4).unwrap();
        puzzle.apply_immediate_move((3, 2));

        let calculator = ManhattanDistance;
        let entropy = calculator.calculate(&puzzle);
        assert_eq!(entropy, 1); // One tile moved one space
    }

    #[test]
    fn test_shortest_path_heuristic() {
        let mut puzzle = PuzzleState::new(4).unwrap();
        puzzle.apply_immediate_move((3, 2));

        let calculator = ShortestPathHeuristic;
        let entropy = calculator.calculate(&puzzle);
        assert!(entropy >= 1); // At least as much as Manhattan
    }

    #[test]
    fn test_difficulty_thresholds() {
        assert_eq!(Difficulty::Easy.min_entropy(4), 8);
        assert_eq!(Difficulty::Medium.min_entropy(4), 16);
        assert_eq!(Difficulty::Hard.min_entropy(4), 32);
    }

    #[test]
    fn test_both_calculators_agree_on_solved() {
        let puzzle = PuzzleState::new(4).unwrap();
        let manhattan = ManhattanDistance;
        let shortest = ShortestPathHeuristic;

        assert_eq!(manhattan.calculate(&puzzle), 0);
        assert_eq!(shortest.calculate(&puzzle), 0);
    }

    #[test]
    fn test_entropy_increases_with_moves() {
        let mut puzzle = PuzzleState::new(4).unwrap();
        let calculator = ManhattanDistance;

        let initial = calculator.calculate(&puzzle);
        puzzle.apply_immediate_move((3, 2));
        let after_one = calculator.calculate(&puzzle);
        puzzle.apply_immediate_move((2, 2));
        let after_two = calculator.calculate(&puzzle);

        assert!(after_one > initial);
        assert!(after_two > after_one);
    }
}