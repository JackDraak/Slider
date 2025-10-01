use super::error::PuzzleError;
use super::move_validator::{MoveValidator, Position};
use super::tile::Tile;

/// Constants for puzzle size limits
pub const MIN_SIZE: usize = 3;
pub const MAX_SIZE: usize = 15;

/// Represents the current state of the puzzle
#[derive(Debug, Clone)]
pub struct PuzzleState {
    /// Grid of tiles (None represents the empty cell)
    grid: Vec<Vec<Option<Tile>>>,
    /// Position of the empty cell (row, col)
    empty_pos: Position,
    /// Size of the grid (n x n)
    size: usize,
}

impl PuzzleState {
    /// Creates a new puzzle in solved state
    ///
    /// # Errors
    ///
    /// Returns `PuzzleError::SizeTooSmall` if size < 3
    /// Returns `PuzzleError::SizeTooLarge` if size > 15
    pub fn new(size: usize) -> Result<Self, PuzzleError> {
        if size < MIN_SIZE {
            return Err(PuzzleError::SizeTooSmall {
                size,
                min: MIN_SIZE,
            });
        }
        if size > MAX_SIZE {
            return Err(PuzzleError::SizeTooLarge {
                size,
                max: MAX_SIZE,
            });
        }

        let mut grid = Vec::with_capacity(size);
        let mut tile_number = 1u32;

        for row in 0..size {
            let mut row_vec = Vec::with_capacity(size);
            for col in 0..size {
                if row == size - 1 && col == size - 1 {
                    // Last cell is empty
                    row_vec.push(None);
                } else {
                    row_vec.push(Some(Tile::new_numeric(tile_number, (row, col))));
                    tile_number += 1;
                }
            }
            grid.push(row_vec);
        }

        Ok(Self {
            grid,
            empty_pos: (size - 1, size - 1),
            size,
        })
    }

    /// Returns the grid size
    pub fn size(&self) -> usize {
        self.size
    }

    /// Returns the position of the empty cell
    pub fn empty_position(&self) -> Position {
        self.empty_pos
    }

    /// Returns a reference to the tile at the given position
    pub fn tile_at(&self, pos: Position) -> Option<&Tile> {
        let (row, col) = pos;
        self.grid.get(row).and_then(|r| r.get(col)).and_then(|t| t.as_ref())
    }

    /// Returns the current position of a specific tile (by its home position)
    pub fn find_tile_position(&self, home_pos: Position) -> Option<Position> {
        for (row, row_vec) in self.grid.iter().enumerate() {
            for (col, cell) in row_vec.iter().enumerate() {
                if let Some(tile) = cell {
                    if tile.home_position == home_pos {
                        return Some((row, col));
                    }
                }
            }
        }
        None
    }

    /// Performs an immediate move: swaps the tile at `from` with the empty cell
    /// Returns true if the move was successful
    pub fn apply_immediate_move(&mut self, from: Position) -> bool {
        // Size is guaranteed valid since PuzzleState was constructed successfully
        let validator = MoveValidator::new(self.size).expect("valid size");

        if !validator.is_adjacent(from, self.empty_pos) {
            return false;
        }

        let (from_row, from_col) = from;
        let (empty_row, empty_col) = self.empty_pos;

        // Swap tile with empty cell
        let tile = self.grid[from_row][from_col].take();
        self.grid[empty_row][empty_col] = tile;
        self.grid[from_row][from_col] = None;

        // Update empty position
        self.empty_pos = from;

        true
    }

    /// Applies a chain move by executing a sequence of immediate moves
    pub fn apply_chain_move(&mut self, target: Position) -> bool {
        // Size is guaranteed valid since PuzzleState was constructed successfully
        let validator = MoveValidator::new(self.size).expect("valid size");

        if let Some(moves) = validator.resolve_chain_move(target, self.empty_pos) {
            for move_pos in moves {
                if !self.apply_immediate_move(move_pos) {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }

    /// Checks if the puzzle is in the solved state
    pub fn is_solved(&self) -> bool {
        for (row, row_vec) in self.grid.iter().enumerate() {
            for (col, cell) in row_vec.iter().enumerate() {
                if let Some(tile) = cell {
                    if tile.home_position != (row, col) {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Returns an iterator over all tiles and their current positions
    pub fn tiles(&self) -> impl Iterator<Item = (Position, &Tile)> {
        self.grid.iter().enumerate().flat_map(|(row, row_vec)| {
            row_vec.iter().enumerate().filter_map(move |(col, cell)| {
                cell.as_ref().map(|tile| ((row, col), tile))
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_puzzle_is_solved() {
        let puzzle = PuzzleState::new(4).unwrap();
        assert!(puzzle.is_solved());
        assert_eq!(puzzle.empty_position(), (3, 3));
    }

    #[test]
    fn test_puzzle_size() {
        let puzzle = PuzzleState::new(5).unwrap();
        assert_eq!(puzzle.size(), 5);
    }

    #[test]
    fn test_invalid_size_too_small() {
        let result = PuzzleState::new(2);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PuzzleError::SizeTooSmall { .. }));
    }

    #[test]
    fn test_invalid_size_too_large() {
        let result = PuzzleState::new(23);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PuzzleError::SizeTooLarge { .. }));
    }

    #[test]
    fn test_tile_at() {
        let puzzle = PuzzleState::new(4).unwrap();
        let tile = puzzle.tile_at((0, 0)).unwrap();
        assert_eq!(tile.numeric_value(), Some(1));
        assert_eq!(puzzle.tile_at((3, 3)), None); // Empty cell
    }

    #[test]
    fn test_immediate_move() {
        let mut puzzle = PuzzleState::new(4).unwrap();
        assert!(puzzle.apply_immediate_move((3, 2)));
        assert_eq!(puzzle.empty_position(), (3, 2));
        assert_eq!(puzzle.tile_at((3, 3)).unwrap().numeric_value(), Some(15));
        assert!(!puzzle.is_solved());
    }

    #[test]
    fn test_invalid_immediate_move() {
        let mut puzzle = PuzzleState::new(4).unwrap();
        assert!(!puzzle.apply_immediate_move((0, 0))); // Not adjacent
    }

    #[test]
    fn test_chain_move() {
        let mut puzzle = PuzzleState::new(4).unwrap();
        // Move tile at (3, 0) when empty is at (3, 3)
        assert!(puzzle.apply_chain_move((3, 0)));
        assert_eq!(puzzle.empty_position(), (3, 0));
    }

    #[test]
    fn test_find_tile_position() {
        let mut puzzle = PuzzleState::new(4).unwrap();
        puzzle.apply_immediate_move((3, 2));
        // Tile with home (3, 2) should now be at (3, 3)
        assert_eq!(puzzle.find_tile_position((3, 2)), Some((3, 3)));
    }
}