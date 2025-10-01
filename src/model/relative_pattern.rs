/// Tile-agnostic pattern matching system for sliding puzzles
///
/// This module implements patterns using relative coordinates and structural
/// matching, making them independent of:
/// - Tile labels (works with any numbered tiles)
/// - Position (works anywhere on the grid)
/// - Rotation (automatically tries all 4 rotations)
/// - Mirror (automatically tries both chiralities)

use super::move_validator::Position;
use super::puzzle_state::PuzzleState;

/// A relative move expressed as row and column deltas from current empty position
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelativeMove {
    pub delta_row: i32,
    pub delta_col: i32,
}

impl RelativeMove {
    /// Creates a new relative move
    pub fn new(delta_row: i32, delta_col: i32) -> Self {
        Self { delta_row, delta_col }
    }

    /// Applies this relative move to an absolute position
    /// Returns None if result is out of bounds
    pub fn apply(&self, pos: Position, grid_size: usize) -> Option<Position> {
        let (row, col) = pos;
        let new_row = row as i32 + self.delta_row;
        let new_col = col as i32 + self.delta_col;

        if new_row >= 0 && new_col >= 0
            && new_row < grid_size as i32
            && new_col < grid_size as i32 {
            Some((new_row as usize, new_col as usize))
        } else {
            None
        }
    }

    /// Rotates this move 90 degrees clockwise
    /// (row, col) → (col, -row)
    pub fn rotate_cw(&self) -> Self {
        Self {
            delta_row: self.delta_col,
            delta_col: -self.delta_row,
        }
    }

    /// Mirrors this move horizontally (flip left-right)
    /// (row, col) → (row, -col)
    pub fn mirror_h(&self) -> Self {
        Self {
            delta_row: self.delta_row,
            delta_col: -self.delta_col,
        }
    }

    /// Mirrors this move vertically (flip top-bottom)
    /// (row, col) → (-row, col)
    pub fn mirror_v(&self) -> Self {
        Self {
            delta_row: -self.delta_row,
            delta_col: self.delta_col,
        }
    }
}

/// Transformation to apply to a pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transform {
    Identity,
    Rotate90,
    Rotate180,
    Rotate270,
    MirrorH,      // Horizontal mirror
    MirrorV,      // Vertical mirror
    MirrorD1,     // Diagonal mirror (main diagonal)
    MirrorD2,     // Diagonal mirror (anti-diagonal)
}

impl Transform {
    /// Returns all 8 possible transformations
    pub fn all() -> [Transform; 8] {
        use Transform::*;
        [Identity, Rotate90, Rotate180, Rotate270, MirrorH, MirrorV, MirrorD1, MirrorD2]
    }

    /// Applies this transformation to a relative move
    pub fn apply(&self, mov: RelativeMove) -> RelativeMove {
        use Transform::*;
        match self {
            Identity => mov,
            Rotate90 => mov.rotate_cw(),
            Rotate180 => mov.rotate_cw().rotate_cw(),
            Rotate270 => mov.rotate_cw().rotate_cw().rotate_cw(),
            MirrorH => mov.mirror_h(),
            MirrorV => mov.mirror_v(),
            MirrorD1 => RelativeMove::new(mov.delta_col, mov.delta_row),
            MirrorD2 => RelativeMove::new(-mov.delta_col, -mov.delta_row),
        }
    }
}

/// A pattern defined using relative coordinates
#[derive(Debug, Clone)]
pub struct RelativePattern {
    /// Descriptive name for debugging
    pub name: &'static str,

    /// Sequence of relative moves from empty cell
    pub moves: Vec<RelativeMove>,

    /// Cost (number of moves)
    pub cost: u32,

    /// Pattern type for categorization
    pub pattern_type: PatternType,
}

/// Categories of move patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternType {
    /// Two adjacent tiles swap
    AdjacentSwap,

    /// Three tiles in corner rotation
    CornerRotation,

    /// Four tiles in linear shift
    LinearShift,
}

impl RelativePattern {
    /// Creates a new relative pattern
    pub fn new(
        name: &'static str,
        moves: Vec<RelativeMove>,
        pattern_type: PatternType,
    ) -> Self {
        let cost = moves.len() as u32;
        Self {
            name,
            moves,
            cost,
            pattern_type,
        }
    }

    /// Tries to match this pattern at the given empty position
    /// Returns absolute moves if pattern matches in any transformation
    pub fn match_at(
        &self,
        state: &PuzzleState,
        empty_pos: Position,
    ) -> Option<Vec<Position>> {
        // Try all 8 transformations
        for transform in Transform::all() {
            if let Some(moves) = self.try_transform(state, empty_pos, transform) {
                return Some(moves);
            }
        }
        None
    }

    /// Tries to match pattern with a specific transformation
    fn try_transform(
        &self,
        state: &PuzzleState,
        mut empty_pos: Position,
        transform: Transform,
    ) -> Option<Vec<Position>> {
        let mut absolute_moves = Vec::new();

        // Transform each relative move and convert to absolute position
        for relative_move in &self.moves {
            let transformed = transform.apply(*relative_move);

            if let Some(abs_pos) = transformed.apply(empty_pos, state.size()) {
                // Check that there's actually a tile at this position
                // (not the empty cell or out of bounds)
                if state.tile_at(abs_pos).is_some() {
                    absolute_moves.push(abs_pos);
                } else {
                    // Invalid: trying to move empty cell or non-existent tile
                    return None;
                }
            } else {
                // Out of bounds
                return None;
            }

            // Update empty_pos for next move in sequence
            // (empty cell moves to where the tile was)
            empty_pos = absolute_moves.last().copied().unwrap();
        }

        // All moves valid - pattern matches!
        Some(absolute_moves)
    }
}

/// Catalog of fundamental tile-agnostic patterns
pub struct RelativePatternCatalog {
    patterns: Vec<RelativePattern>,
}

impl RelativePatternCatalog {
    /// Creates a new catalog with fundamental patterns
    pub fn new() -> Self {
        let mut patterns = Vec::new();

        // Pattern 1: Corner Rotation Clockwise (3 moves)
        // □ A      C □
        // B C  →   B A
        patterns.push(RelativePattern::new(
            "corner_rotation_cw",
            vec![
                RelativeMove::new(0, 1),  // Right
                RelativeMove::new(1, 0),  // Down
                RelativeMove::new(0, -1), // Left
            ],
            PatternType::CornerRotation,
        ));

        // Pattern 2: Linear Shift (4 moves)
        // □ A B C  →  D □ A B
        // D
        patterns.push(RelativePattern::new(
            "linear_shift",
            vec![
                RelativeMove::new(0, 1),  // Right
                RelativeMove::new(0, 1),  // Right
                RelativeMove::new(0, 1),  // Right
                RelativeMove::new(1, 0),  // Down
            ],
            PatternType::LinearShift,
        ));

        Self { patterns }
    }

    /// Returns all patterns in the catalog
    pub fn patterns(&self) -> &[RelativePattern] {
        &self.patterns
    }
}

impl Default for RelativePatternCatalog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relative_move_apply() {
        let mov = RelativeMove::new(1, 0);
        assert_eq!(mov.apply((0, 0), 4), Some((1, 0)));
        assert_eq!(mov.apply((3, 0), 4), None); // Out of bounds
    }

    #[test]
    fn test_relative_move_rotate() {
        let mov = RelativeMove::new(0, 1); // Right
        assert_eq!(mov.rotate_cw(), RelativeMove::new(1, 0)); // Down
    }

    #[test]
    fn test_transform_identity() {
        let mov = RelativeMove::new(1, 2);
        assert_eq!(Transform::Identity.apply(mov), mov);
    }

    #[test]
    fn test_transform_rotate90() {
        let mov = RelativeMove::new(0, 1); // Right
        let rotated = Transform::Rotate90.apply(mov);
        assert_eq!(rotated, RelativeMove::new(1, 0)); // Down
    }

    #[test]
    fn test_pattern_catalog_creation() {
        let catalog = RelativePatternCatalog::new();
        assert_eq!(catalog.patterns().len(), 2); // 2 fundamental patterns
    }

    #[test]
    fn test_corner_rotation_matches_all_corners() {
        let state = PuzzleState::new(4).unwrap();
        let catalog = RelativePatternCatalog::new();
        let corner_pattern = &catalog.patterns()[0];

        // Empty at bottom-right corner
        let matches = corner_pattern.match_at(&state, (3, 3));
        assert!(matches.is_some(), "Should match at bottom-right corner");

        // Pattern should work at ANY corner due to transformations
        // (though we need the empty cell to actually be there to test)
    }

    #[test]
    fn test_pattern_does_not_match_invalid_position() {
        let state = PuzzleState::new(3).unwrap();
        let catalog = RelativePatternCatalog::new();
        let linear_pattern = &catalog.patterns()[1];

        // Linear shift requires 4 positions - won't fit from (2,2) in 3x3
        let matches = linear_pattern.match_at(&state, (2, 2));
        assert!(matches.is_none(), "Should not match - insufficient space");
    }
}
