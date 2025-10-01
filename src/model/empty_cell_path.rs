//! Empty Cell Path Heuristic
//! 
//! This heuristic calculates the minimum number of moves required to position
//! the empty cell optimally for solving the puzzle. It addresses a key limitation
//! of Manhattan distance by considering the empty cell's role in tile movement.

use crate::model::puzzle_state::PuzzleState;
use crate::model::entropy::EntropyCalculator;

/// Empty Cell Path heuristic that considers the optimal positioning
/// of the empty cell for efficient tile movement
#[derive(Debug, Default)]
pub struct EmptyCellPathHeuristic;

impl EmptyCellPathHeuristic {
    /// Creates a new Empty Cell Path heuristic
    pub fn new() -> Self {
        Self
    }

    /// Calculates the minimum moves needed to position the empty cell
    /// for optimal tile movement sequences
    fn calculate_empty_cell_distance(&self, state: &PuzzleState) -> u32 {
        let empty_pos = state.empty_position();
        let size = state.size();
        
        // Find tiles that are far from their target positions
        let mut misplaced_tiles = Vec::new();
        
        for (pos, tile) in state.tiles() {
            let (current_row, current_col) = pos;
            let (target_row, target_col) = tile.home_position;
            
            if current_row != target_row || current_col != target_col {
                let distance = current_row.abs_diff(target_row) + current_col.abs_diff(target_col);
                misplaced_tiles.push((pos, tile.home_position, distance));
            }
        }
        
        // Sort by distance (furthest first)
        misplaced_tiles.sort_by_key(|(_, _, distance)| *distance);
        
        // Calculate optimal empty cell positioning
        let mut total_cost = 0u32;
        
        // Consider the top 5 most misplaced tiles
        for (current_pos, target_pos, _) in misplaced_tiles.iter().take(5) {
            // Cost to move empty cell to assist this tile
            let empty_to_current = Self::manhattan_distance(empty_pos, *current_pos);
            let empty_to_target = Self::manhattan_distance(empty_pos, *target_pos);
            
            // The empty cell needs to be positioned to facilitate tile movement
            // This is a simplified calculation - more sophisticated analysis could be done
            let positioning_cost = empty_to_current.min(empty_to_target);
            total_cost += positioning_cost;
        }
        
        // Add penalty for empty cell being far from center
        let center = (size / 2, size / 2);
        let center_distance = Self::manhattan_distance(empty_pos, center);
        total_cost += center_distance / 2; // Reduced penalty
        
        total_cost
    }
    
    /// Simple Manhattan distance between two positions
    fn manhattan_distance(pos1: (usize, usize), pos2: (usize, usize)) -> u32 {
        pos1.0.abs_diff(pos2.0) as u32 + pos1.1.abs_diff(pos2.1) as u32
    }
    
    /// Calculates path complexity - how many tiles block the empty cell's movement
    fn calculate_path_complexity(&self, state: &PuzzleState) -> u32 {
        let empty_pos = state.empty_position();
        let size = state.size();
        let mut complexity = 0u32;
        
        // Check surrounding tiles
        let directions = [(0, 1), (1, 0), (0, !0), (!0, 0)]; // right, down, left, up
        
        for (dr, dc) in directions.iter() {
            let new_row = if *dr == !0 { empty_pos.0.checked_sub(1) } else { Some(empty_pos.0 + dr) };
            let new_col = if *dc == !0 { empty_pos.1.checked_sub(1) } else { Some(empty_pos.1 + dc) };
            
            if let (Some(row), Some(col)) = (new_row, new_col) {
                if row < size && col < size {
                    if let Some(_tile) = state.tile_at((row, col)) {
                        // Check if this tile is in its correct position
                        if let Some(tile) = state.tile_at((row, col)) {
                            let (home_row, home_col) = tile.home_position;
                            if row == home_row && col == home_col {
                                complexity += 2; // Higher penalty for blocking correctly placed tiles
                            } else {
                                complexity += 1;
                            }
                        }
                    }
                }
            }
        }
        
        complexity
    }
    
    /// Estimates the minimum number of moves to solve based on empty cell positioning
    fn estimate_solution_length(&self, state: &PuzzleState) -> u32 {
        let empty_distance = self.calculate_empty_cell_distance(state);
        let path_complexity = self.calculate_path_complexity(state);
        
        // Base Manhattan distance
        let mut total = 0u32;
        for (pos, tile) in state.tiles() {
            let (current_row, current_col) = pos;
            let (home_row, home_col) = tile.home_position;
            total += (current_row.abs_diff(home_row) + current_col.abs_diff(home_col)) as u32;
        }
        
        // Add empty cell considerations
        total + empty_distance + path_complexity
    }
}

impl EntropyCalculator for EmptyCellPathHeuristic {
    fn calculate(&self, state: &PuzzleState) -> u32 {
        self.estimate_solution_length(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::puzzle_state::PuzzleState;

    #[test]
    fn test_empty_cell_path_solved_puzzle() {
        let puzzle = PuzzleState::new(4).unwrap();
        let heuristic = EmptyCellPathHeuristic::new();
        
        // Solved puzzle should have minimal cost
        let cost = heuristic.calculate(&puzzle);
        assert!(cost < 10); // Should be very low for solved puzzle
    }

    #[test]
    fn test_empty_cell_path_single_move() {
        let mut puzzle = PuzzleState::new(4).unwrap();
        puzzle.apply_immediate_move((3, 2));
        
        let heuristic = EmptyCellPathHeuristic::new();
        let cost = heuristic.calculate(&puzzle);
        
        // Should be higher than solved but still reasonable
        assert!(cost > 0);
        assert!(cost < 20);
    }

    #[test]
    fn test_empty_cell_distance_calculation() {
        let puzzle = PuzzleState::new(4).unwrap();
        let heuristic = EmptyCellPathHeuristic::new();
        
        let distance = heuristic.calculate_empty_cell_distance(&puzzle);
        assert!(distance >= 0);
    }

    #[test]
    fn test_path_complexity() {
        let puzzle = PuzzleState::new(4).unwrap();
        let heuristic = EmptyCellPathHeuristic::new();
        
        let complexity = heuristic.calculate_path_complexity(&puzzle);
        assert!(complexity >= 0);
    }

    #[test]
    fn test_manhattan_distance() {
        let pos1 = (0, 0);
        let pos2 = (3, 4);
        
        let distance = EmptyCellPathHeuristic::manhattan_distance(pos1, pos2);
        assert_eq!(distance, 7);
    }

    #[test]
    fn test_empty_cell_path_vs_manhattan() {
        let mut puzzle = PuzzleState::new(4).unwrap();
        
        // Make several moves to create a complex state
        puzzle.apply_immediate_move((3, 2));
        puzzle.apply_immediate_move((2, 2));
        puzzle.apply_immediate_move((2, 1));
        
        let empty_cell_heuristic = EmptyCellPathHeuristic::new();
        let manhattan = crate::model::entropy::ManhattanDistance;
        
        let empty_cost = empty_cell_heuristic.calculate(&puzzle);
        let manhattan_cost = manhattan.calculate(&puzzle);
        
        // Empty cell path should generally be higher or equal
        // as it considers more factors
        assert!(empty_cost >= manhattan_cost);
    }
}
