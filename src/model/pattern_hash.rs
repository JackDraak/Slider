/// Fast pattern matching using precomputed hash tables
///
/// Instead of trying all 8 transformations for each pattern at runtime,
/// we precompute a lookup table that maps local tile configurations
/// to matching patterns and their transformations.

use super::move_validator::Position;
use super::puzzle_state::PuzzleState;
use super::relative_pattern::{RelativeMove, RelativePattern, Transform};
use std::collections::HashMap;

/// A local configuration around the empty cell
/// Encoded as which adjacent positions have tiles (bitmask)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct LocalConfig {
    /// Bitmask: bit 0 = up, bit 1 = down, bit 2 = left, bit 3 = right
    /// bit 4 = up-left, bit 5 = up-right, bit 6 = down-left, bit 7 = down-right
    adjacent_mask: u8,
}

impl LocalConfig {
    /// Creates a local configuration from the puzzle state at empty position
    fn from_state(state: &PuzzleState, empty_pos: Position) -> Self {
        let (empty_row, empty_col) = empty_pos;
        let size = state.size();
        let mut mask = 0u8;

        // Check 4 cardinal directions
        let checks = [
            (-1, 0, 0),  // Up
            (1, 0, 1),   // Down
            (0, -1, 2),  // Left
            (0, 1, 3),   // Right
            (-1, -1, 4), // Up-left
            (-1, 1, 5),  // Up-right
            (1, -1, 6),  // Down-left
            (1, 1, 7),   // Down-right
        ];

        for (dr, dc, bit) in checks {
            let r = empty_row as i32 + dr;
            let c = empty_col as i32 + dc;

            if r >= 0 && c >= 0 && r < size as i32 && c < size as i32 {
                let pos = (r as usize, c as usize);
                if state.tile_at(pos).is_some() {
                    mask |= 1 << bit;
                }
            }
        }

        Self { adjacent_mask: mask }
    }
}

/// A matched pattern with its transformation and absolute moves
#[derive(Debug, Clone)]
pub struct PatternMatch {
    pub pattern_name: String,
    pub moves: Vec<Position>,
    pub cost: u32,
}

/// Fast pattern matcher using precomputed hash table
pub struct PatternHashTable {
    /// Maps local configuration to all matching patterns
    lookup: HashMap<LocalConfig, Vec<(usize, Transform, Vec<RelativeMove>)>>,

    /// Original patterns for reference
    patterns: Vec<RelativePattern>,
}

impl PatternHashTable {
    /// Builds a hash table from a set of patterns
    pub fn from_patterns(patterns: Vec<RelativePattern>) -> Self {
        let mut lookup: HashMap<LocalConfig, Vec<(usize, Transform, Vec<RelativeMove>)>> = HashMap::new();

        // For each pattern
        for (pattern_idx, pattern) in patterns.iter().enumerate() {
            // Try all 8 transformations
            for transform in Transform::all() {
                // Apply transformation to all moves in pattern
                let transformed_moves: Vec<RelativeMove> = pattern.moves
                    .iter()
                    .map(|&m| transform.apply(m))
                    .collect();

                // Determine what local configuration this requires
                if let Some(config) = Self::required_config(&transformed_moves) {
                    lookup.entry(config)
                        .or_insert_with(Vec::new)
                        .push((pattern_idx, transform, transformed_moves));
                }
            }
        }

        Self { lookup, patterns }
    }

    /// Determines what local configuration is required for a pattern
    /// Just checks that first move direction has a tile
    fn required_config(moves: &[RelativeMove]) -> Option<LocalConfig> {
        if moves.is_empty() {
            return None;
        }

        let mut mask = 0u8;

        // Only care about the first move - need a tile there
        let first = moves[0];
        let bit = match (first.delta_row, first.delta_col) {
            (-1, 0) => Some(0),   // Up
            (1, 0) => Some(1),    // Down
            (0, -1) => Some(2),   // Left
            (0, 1) => Some(3),    // Right
            (-1, -1) => Some(4),  // Up-left
            (-1, 1) => Some(5),   // Up-right
            (1, -1) => Some(6),   // Down-left
            (1, 1) => Some(7),    // Down-right
            _ => return None,     // Multi-step move
        };

        if let Some(b) = bit {
            mask |= 1 << b;
            Some(LocalConfig { adjacent_mask: mask })
        } else {
            None
        }
    }

    /// Fast pattern matching: O(1) hash lookup instead of O(patterns × 8)
    pub fn match_at(&self, state: &PuzzleState, empty_pos: Position) -> Vec<PatternMatch> {
        let config = LocalConfig::from_state(state, empty_pos);
        let mut matches = Vec::new();

        // Check all entries where required bits are present
        for (required_config, candidates) in &self.lookup {
            // Check if config has all required bits set
            if (config.adjacent_mask & required_config.adjacent_mask) == required_config.adjacent_mask {
            for (pattern_idx, _transform, transformed_moves) in candidates {
                // Verify the pattern actually applies (bounds checking)
                let mut absolute_moves = Vec::new();
                let mut current_empty = empty_pos;
                let mut valid = true;

                for rel_move in transformed_moves {
                    if let Some(abs_pos) = rel_move.apply(current_empty, state.size()) {
                        if state.tile_at(abs_pos).is_some() {
                            absolute_moves.push(abs_pos);
                            current_empty = abs_pos;
                        } else {
                            valid = false;
                            break;
                        }
                    } else {
                        valid = false;
                        break;
                    }
                }

                if valid {
                    matches.push(PatternMatch {
                        pattern_name: self.patterns[*pattern_idx].name.to_string(),
                        moves: absolute_moves,
                        cost: self.patterns[*pattern_idx].cost,
                    });
                }
            }
            }
        }

        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::relative_pattern::{PatternType, RelativePatternCatalog};

    #[test]
    fn test_local_config_from_state() {
        let state = PuzzleState::new(3).unwrap();
        // Empty at (2, 2) has tiles up and left
        let config = LocalConfig::from_state(&state, (2, 2));

        // Bits: up=1, left=1, others=0 (because edges/corners)
        assert_ne!(config.adjacent_mask, 0);
    }

    #[test]
    fn test_pattern_hash_table_creation() {
        let catalog = RelativePatternCatalog::new();
        let patterns = catalog.patterns().to_vec();

        let hash_table = PatternHashTable::from_patterns(patterns);
        assert!(!hash_table.lookup.is_empty());
    }

    #[test]
    fn test_fast_pattern_matching() {
        let catalog = RelativePatternCatalog::new();
        let patterns = catalog.patterns().to_vec();
        let hash_table = PatternHashTable::from_patterns(patterns);

        let state = PuzzleState::new(4).unwrap();
        let matches = hash_table.match_at(&state, (3, 3));

        // Should find at least one match at bottom-right corner
        assert!(!matches.is_empty(), "Should find patterns at corner");
    }

    #[test]
    fn test_hash_table_finds_valid_patterns() {
        let catalog = RelativePatternCatalog::new();
        let patterns_vec = catalog.patterns().to_vec();
        let hash_table = PatternHashTable::from_patterns(patterns_vec);

        let mut state = PuzzleState::new(4).unwrap();
        let matches = hash_table.match_at(&state, (3, 3));

        // Verify all returned matches are valid
        for pattern_match in matches {
            let mut test_state = state.clone();
            for move_pos in &pattern_match.moves {
                assert!(test_state.apply_immediate_move(*move_pos),
                    "Pattern match contains invalid move");
            }
        }
    }
}
