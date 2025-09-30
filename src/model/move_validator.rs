use super::error::PuzzleError;
use super::puzzle_state::{MAX_SIZE, MIN_SIZE};
use std::collections::HashSet;

/// Represents a position in the grid (row, col)
pub type Position = (usize, usize);

/// Validates moves based on empty cell position and grid constraints
pub struct MoveValidator {
    grid_size: usize,
}

impl MoveValidator {
    /// Creates a new move validator
    ///
    /// # Errors
    ///
    /// Returns `PuzzleError` if grid_size is invalid (< 3 or > 22)
    pub fn new(grid_size: usize) -> Result<Self, PuzzleError> {
        if grid_size < MIN_SIZE {
            return Err(PuzzleError::SizeTooSmall {
                size: grid_size,
                min: MIN_SIZE,
            });
        }
        if grid_size > MAX_SIZE {
            return Err(PuzzleError::SizeTooLarge {
                size: grid_size,
                max: MAX_SIZE,
            });
        }
        Ok(Self { grid_size })
    }

    /// Returns all tiles that can immediately move into the empty position
    /// Truth table:
    /// - Corner: 2 immediate moves
    /// - Edge: 3 immediate moves
    /// - Surrounded: 4 immediate moves
    pub fn get_immediate_moves(&self, empty_pos: Position) -> Vec<Position> {
        let (row, col) = empty_pos;
        let mut moves = Vec::new();

        // Up
        if row > 0 {
            moves.push((row - 1, col));
        }
        // Down
        if row < self.grid_size - 1 {
            moves.push((row + 1, col));
        }
        // Left
        if col > 0 {
            moves.push((row, col - 1));
        }
        // Right
        if col < self.grid_size - 1 {
            moves.push((row, col + 1));
        }

        moves
    }

    /// Returns all positions that can move to empty (immediate + chain destinations)
    /// Includes tiles in line with the empty cell
    pub fn get_all_legal_moves(&self, empty_pos: Position) -> Vec<Position> {
        let (empty_row, empty_col) = empty_pos;
        let mut legal = HashSet::new();

        // Add immediate moves
        for pos in self.get_immediate_moves(empty_pos) {
            legal.insert(pos);
        }

        // Add chain move destinations (tiles in same row or column)
        // Horizontal chain
        for col in 0..self.grid_size {
            if col != empty_col {
                legal.insert((empty_row, col));
            }
        }
        // Vertical chain
        for row in 0..self.grid_size {
            if row != empty_row {
                legal.insert((row, empty_col));
            }
        }

        legal.into_iter().collect()
    }

    /// Determines if a position is adjacent to the empty cell
    pub fn is_adjacent(&self, pos: Position, empty_pos: Position) -> bool {
        let (row, col) = pos;
        let (empty_row, empty_col) = empty_pos;

        (row == empty_row && col.abs_diff(empty_col) == 1)
            || (col == empty_col && row.abs_diff(empty_row) == 1)
    }

    /// Determines if a position can legally move (immediate or chain)
    pub fn is_legal_move(&self, pos: Position, empty_pos: Position) -> bool {
        let (row, col) = pos;
        let (empty_row, empty_col) = empty_pos;

        // Same row or same column
        row == empty_row || col == empty_col
    }

    /// Resolves a chain move into a sequence of immediate moves
    /// Returns None if the move is not legal
    pub fn resolve_chain_move(
        &self,
        from: Position,
        empty_pos: Position,
    ) -> Option<Vec<Position>> {
        if !self.is_legal_move(from, empty_pos) {
            return None;
        }

        if self.is_adjacent(from, empty_pos) {
            // Immediate move
            return Some(vec![from]);
        }

        let (from_row, from_col) = from;
        let (empty_row, empty_col) = empty_pos;
        let mut moves = Vec::new();

        if from_row == empty_row {
            // Horizontal chain
            let start = from_col.min(empty_col);
            let end = from_col.max(empty_col);

            if from_col < empty_col {
                // Moving left to right: tiles shift left
                for col in (start..end).rev() {
                    moves.push((from_row, col));
                }
            } else {
                // Moving right to left: tiles shift right
                for col in (start + 1)..=end {
                    moves.push((from_row, col));
                }
            }
        } else if from_col == empty_col {
            // Vertical chain
            let start = from_row.min(empty_row);
            let end = from_row.max(empty_row);

            if from_row < empty_row {
                // Moving up to down: tiles shift up
                for row in (start..end).rev() {
                    moves.push((row, from_col));
                }
            } else {
                // Moving down to up: tiles shift down
                for row in (start + 1)..=end {
                    moves.push((row, from_col));
                }
            }
        }

        Some(moves)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_corner_position_immediate_moves() {
        let validator = MoveValidator::new(4).unwrap();
        let moves = validator.get_immediate_moves((0, 0));
        assert_eq!(moves.len(), 2); // Corner has 2 immediate moves
        assert!(moves.contains(&(0, 1)));
        assert!(moves.contains(&(1, 0)));
    }

    #[test]
    fn test_edge_position_immediate_moves() {
        let validator = MoveValidator::new(4).unwrap();
        let moves = validator.get_immediate_moves((0, 1));
        assert_eq!(moves.len(), 3); // Edge has 3 immediate moves
    }

    #[test]
    fn test_surrounded_position_immediate_moves() {
        let validator = MoveValidator::new(4).unwrap();
        let moves = validator.get_immediate_moves((1, 1));
        assert_eq!(moves.len(), 4); // Surrounded has 4 immediate moves
    }

    #[test]
    fn test_is_adjacent() {
        let validator = MoveValidator::new(4).unwrap();
        assert!(validator.is_adjacent((1, 1), (1, 2)));
        assert!(validator.is_adjacent((1, 1), (2, 1)));
        assert!(!validator.is_adjacent((1, 1), (2, 2)));
        assert!(!validator.is_adjacent((1, 1), (3, 1)));
    }

    #[test]
    fn test_chain_move_horizontal() {
        let validator = MoveValidator::new(4).unwrap();
        // Empty at (0, 3), click tile at (0, 0)
        let moves = validator.resolve_chain_move((0, 0), (0, 3)).unwrap();
        assert_eq!(moves, vec![(0, 2), (0, 1), (0, 0)]);
    }

    #[test]
    fn test_chain_move_vertical() {
        let validator = MoveValidator::new(4).unwrap();
        // Empty at (3, 0), click tile at (0, 0)
        let moves = validator.resolve_chain_move((0, 0), (3, 0)).unwrap();
        assert_eq!(moves, vec![(2, 0), (1, 0), (0, 0)]);
    }

    #[test]
    fn test_immediate_move() {
        let validator = MoveValidator::new(4).unwrap();
        let moves = validator.resolve_chain_move((1, 2), (1, 3)).unwrap();
        assert_eq!(moves, vec![(1, 2)]); // Single immediate move
    }

    #[test]
    fn test_illegal_move() {
        let validator = MoveValidator::new(4).unwrap();
        let moves = validator.resolve_chain_move((1, 1), (2, 2));
        assert!(moves.is_none()); // Diagonal not allowed
    }
}