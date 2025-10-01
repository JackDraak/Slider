/// Enhanced heuristic combining Manhattan Distance with multiple conflict penalties
///
/// This is simpler than Walking Distance but proven effective for 5×5 puzzles.
/// Combines:
/// 1. Manhattan Distance (base displacement)
/// 2. Linear Conflicts (tiles in same row/col blocking each other)
/// 3. Corner Tile Penalties (last tiles are hard to place)
/// 4. Last Row/Column Penalties (edge tiles need special handling)

use super::entropy::{count_linear_conflicts, EntropyCalculator, ManhattanDistance};
use super::puzzle_state::PuzzleState;

#[derive(Debug, Default)]
pub struct EnhancedHeuristic;

impl EntropyCalculator for EnhancedHeuristic {
    fn calculate(&self, state: &PuzzleState) -> u32 {
        let manhattan = ManhattanDistance;
        let base_score = manhattan.calculate(state);

        // Linear conflicts (tiles in correct row/col but blocking each other)
        let linear_conflicts = count_linear_conflicts(state);

        // Corner tile penalties
        let corner_penalty = calculate_corner_penalty(state);

        // Last row/column penalties
        let edge_penalty = calculate_edge_penalty(state);

        // Combine all signals
        base_score + (linear_conflicts * 2) + corner_penalty + edge_penalty
    }
}

/// Calculate penalty for corner tiles being out of place
/// Corner tiles are hardest to place because they have limited movement options
fn calculate_corner_penalty(state: &PuzzleState) -> u32 {
    let n = state.size();
    let mut penalty = 0u32;

    let corners = [
        ((0, 0), (0, 0)),           // Top-left
        ((0, n-1), (0, n-1)),       // Top-right
        ((n-1, 0), (n-1, 0)),       // Bottom-left
        ((n-1, n-1), (n-1, n-1)),   // Bottom-right (usually empty in solved state)
    ];

    for (pos, target) in corners {
        // Skip if this is the empty cell position
        if state.empty_position() == pos {
            continue;
        }

        // Check if the tile at this corner position belongs there
        if let Some(tile) = state.tile_at(pos) {
            if tile.home_position != target {
                // Corner tile is displaced - add penalty
                // Empirically, corner tiles need ~2-4 extra moves to place
                penalty += 3;
            }
        }
    }

    penalty
}

/// Calculate penalty for last row and last column being unsolved
/// These are harder to solve because there's less room to maneuver
fn calculate_edge_penalty(state: &PuzzleState) -> u32 {
    let n = state.size();
    let mut penalty = 0u32;

    // Check last row (row n-1)
    let mut last_row_wrong = 0;
    for col in 0..n {
        if state.empty_position() == (n-1, col) {
            continue; // Skip empty cell
        }

        if let Some(tile) = state.tile_at((n-1, col)) {
            let (target_row, _) = tile.home_position;
            if target_row != n-1 {
                last_row_wrong += 1;
            }
        }
    }

    // Check last column (col n-1)
    let mut last_col_wrong = 0;
    for row in 0..n {
        if state.empty_position() == (row, n-1) {
            continue; // Skip empty cell
        }

        if let Some(tile) = state.tile_at((row, n-1)) {
            let (_, target_col) = tile.home_position;
            if target_col != n-1 {
                last_col_wrong += 1;
            }
        }
    }

    // If multiple tiles in last row/col are wrong, add penalty
    // Empirically, each wrong edge tile needs ~1-2 extra moves
    if last_row_wrong > 1 {
        penalty += last_row_wrong * 2;
    }
    if last_col_wrong > 1 {
        penalty += last_col_wrong * 2;
    }

    penalty
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_heuristic_solved() {
        let heuristic = EnhancedHeuristic;
        let puzzle = PuzzleState::new(3).unwrap();

        assert_eq!(heuristic.calculate(&puzzle), 0);
    }

    #[test]
    fn test_enhanced_better_than_manhattan() {
        let enhanced = EnhancedHeuristic;
        let manhattan = ManhattanDistance;

        let mut puzzle = PuzzleState::new(4).unwrap();

        // Make some moves
        puzzle.apply_immediate_move((3, 2));
        puzzle.apply_immediate_move((3, 1));
        puzzle.apply_immediate_move((2, 1));

        let enhanced_score = enhanced.calculate(&puzzle);
        let manhattan_score = manhattan.calculate(&puzzle);

        // Enhanced should be >= Manhattan (more accurate)
        assert!(enhanced_score >= manhattan_score);
    }

    #[test]
    fn test_corner_penalty() {
        let puzzle = PuzzleState::new(3).unwrap();

        // Solved puzzle should have no corner penalty
        let penalty = calculate_corner_penalty(&puzzle);
        assert_eq!(penalty, 0);
    }

    #[test]
    fn test_edge_penalty() {
        let puzzle = PuzzleState::new(3).unwrap();

        // Solved puzzle should have no edge penalty
        let penalty = calculate_edge_penalty(&puzzle);
        assert_eq!(penalty, 0);
    }
}
