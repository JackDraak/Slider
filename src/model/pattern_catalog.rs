/// Pattern-based A* optimization for sliding puzzle solver
///
/// This module defines common move sequences that appear frequently in optimal
/// solutions. By recognizing these patterns during A* search, we can reduce the
/// branching factor and speed up the search significantly.
///
/// ## Pattern Types
///
/// 1. **Corner Rotations** - 3-move cycles that rotate tiles around corners
/// 2. **Edge Shifts** - Linear sequences along edges
/// 3. **Two-tile Swaps** - Common sequences that effectively swap two tiles
/// 4. **Empty Positioning** - Efficient ways to move empty cell to strategic locations
///
/// ## Implementation Strategy
///
/// Rather than exploring all 2-4 immediate moves at each search node, we:
/// 1. Detect if current state matches a pattern's precondition
/// 2. Apply the entire pattern sequence atomically (as one search step)
/// 3. Adjust g_score by the pattern's move count
/// 4. This reduces branching factor while maintaining optimality

use super::move_validator::Position;

/// Represents a reusable move sequence pattern
#[derive(Debug, Clone)]
pub struct MovePattern {
    /// Descriptive name for debugging
    pub name: &'static str,

    /// Sequence of tile positions to move (in order)
    pub moves: Vec<Position>,

    /// How much this pattern costs (number of moves)
    pub cost: u32,

    /// Precondition checker: does this pattern apply to current state?
    /// (For future enhancement - for now we'll try all patterns)
    pub applies_to: PatternType,
}

/// Categories of move patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternType {
    /// Corner rotation patterns (3-4 moves)
    CornerRotation,

    /// Edge shift patterns (2-3 moves)
    EdgeShift,

    /// Two-tile effective swap (6-8 moves typical)
    TwoTileSwap,

    /// Empty cell repositioning (varies)
    EmptyPositioning,
}

/// Catalog of commonly occurring move patterns
pub struct PatternCatalog {
    patterns: Vec<MovePattern>,
}

impl PatternCatalog {
    /// Creates a new pattern catalog with predefined common patterns
    pub fn new(grid_size: usize) -> Self {
        let mut patterns = Vec::new();

        // Add 3x3 corner rotation patterns
        if grid_size >= 3 {
            patterns.extend(Self::corner_rotations_3x3());
        }

        // Add 4x4 specific patterns
        if grid_size == 4 {
            patterns.extend(Self::common_4x4_patterns());
        }

        Self { patterns }
    }

    /// Returns all patterns in the catalog
    pub fn patterns(&self) -> &[MovePattern] {
        &self.patterns
    }

    /// 3x3 corner rotation patterns
    /// These are the most common multi-move sequences in optimal solutions
    fn corner_rotations_3x3() -> Vec<MovePattern> {
        vec![
            // Top-left corner clockwise rotation (when empty starts at top-left)
            // Pattern: E 1 2     ->    E 1 2    (E moved around corner)
            //          3 4 5     ->    3 4 5
            //          6 7 8     ->    6 7 8
            MovePattern {
                name: "top_left_corner_cw",
                moves: vec![(0, 1), (1, 1), (1, 0)], // right, down, left
                cost: 3,
                applies_to: PatternType::CornerRotation,
            },

            // Top-right corner clockwise
            MovePattern {
                name: "top_right_corner_cw",
                moves: vec![(1, 2), (1, 1), (0, 1)], // down, left, up
                cost: 3,
                applies_to: PatternType::CornerRotation,
            },

            // Bottom-right corner clockwise
            MovePattern {
                name: "bottom_right_corner_cw",
                moves: vec![(2, 1), (1, 1), (1, 2)], // left, up, right
                cost: 3,
                applies_to: PatternType::CornerRotation,
            },

            // Bottom-left corner clockwise
            MovePattern {
                name: "bottom_left_corner_cw",
                moves: vec![(1, 0), (1, 1), (2, 1)], // up, right, down
                cost: 3,
                applies_to: PatternType::CornerRotation,
            },
        ]
    }

    /// Common patterns specific to 4x4 puzzles
    fn common_4x4_patterns() -> Vec<MovePattern> {
        vec![
            // Top row shift right (when empty at top-left)
            MovePattern {
                name: "top_row_shift_right",
                moves: vec![(0, 1), (0, 2)], // Slide tiles left
                cost: 2,
                applies_to: PatternType::EdgeShift,
            },

            // Left column shift down (when empty at top-left)
            MovePattern {
                name: "left_col_shift_down",
                moves: vec![(1, 0), (2, 0)], // Slide tiles up
                cost: 2,
                applies_to: PatternType::EdgeShift,
            },
        ]
    }
}

impl Default for PatternCatalog {
    fn default() -> Self {
        Self::new(4) // Default to 4x4 patterns
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_creation() {
        let catalog = PatternCatalog::new(4);
        assert!(!catalog.patterns().is_empty());
    }

    #[test]
    fn test_3x3_patterns_included() {
        let catalog = PatternCatalog::new(3);
        let corner_patterns: Vec<_> = catalog
            .patterns()
            .iter()
            .filter(|p| p.applies_to == PatternType::CornerRotation)
            .collect();

        assert_eq!(corner_patterns.len(), 4); // 4 corner rotations
    }

    #[test]
    fn test_pattern_costs() {
        let catalog = PatternCatalog::new(4);
        for pattern in catalog.patterns() {
            assert_eq!(pattern.cost, pattern.moves.len() as u32);
        }
    }
}
