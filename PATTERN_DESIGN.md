# Tile-Agnostic Pattern Design Specification

## Problem Statement

Current pattern implementation violates fundamental pattern invariances:
- ❌ Patterns use absolute positions, not relative moves
- ❌ Patterns don't recognize tile-label independence
- ❌ Patterns aren't rotation/mirror invariant
- ❌ 4 corner patterns are really the same pattern at different locations

## Correct Pattern Specification

### Pattern Invariances

A pattern should be recognized **regardless of**:

1. **Tile Labels**: Swapping tiles (3,7) is the same pattern as swapping (1,5)
2. **Position**: Corner rotation at (0,0) is same pattern as corner rotation at (7,7)
3. **Rotation**: Horizontal swap = Vertical swap (90° rotation)
4. **Mirror**: Left-to-right swap = Right-to-left swap (reflection)

### Pattern Representation

Instead of absolute positions, patterns should be:

```rust
struct RelativePattern {
    /// Relative moves from empty cell position
    /// Example: [(0, 1), (1, 0), (0, -1)] means right, down, left
    relative_moves: Vec<(i32, i32)>,

    /// Structural requirement (what tiles must exist)
    /// Example: "3 tiles forming L-shape around empty cell"
    structure: PatternStructure,

    /// Cost (number of moves)
    cost: u32,
}

enum PatternStructure {
    /// Two adjacent tiles (for swap)
    AdjacentPair { direction: RelativeDirection },

    /// Three tiles forming corner (for rotation)
    CornerTriple { orientation: CornerOrientation },

    /// Four tiles in line (for shift)
    LinearQuad { axis: Axis },
}
```

### Example: Corner Rotation Pattern

**Current (wrong)**:
```rust
// 4 separate patterns for 4 corners!
"top_left":     vec![(0, 1), (1, 1), (1, 0)]
"top_right":    vec![(1, 2), (1, 1), (0, 1)]
"bottom_right": vec![(2, 1), (1, 1), (1, 2)]
"bottom_left":  vec![(1, 0), (1, 1), (2, 1)]
```

**Correct (tile-agnostic)**:
```rust
// ONE pattern that matches any corner
CornerRotationCW {
    relative_moves: vec![
        (0, 1),   // Right from empty
        (1, 0),   // Down from new empty
        (0, -1),  // Left from new empty
    ],
    structure: PatternStructure::CornerTriple {
        orientation: Any  // Matches all 4 corners
    },
    cost: 3,
}
```

### Pattern Matching Algorithm

```rust
fn matches_pattern(
    &self,
    state: &PuzzleState,
    empty_pos: Position,
    pattern: &RelativePattern
) -> Option<Vec<Position>> {
    let (empty_row, empty_col) = empty_pos;
    let mut absolute_moves = Vec::new();

    // Try all rotations and mirrors of the pattern
    for transform in [
        Identity, Rotate90, Rotate180, Rotate270,
        MirrorH, MirrorV, MirrorD1, MirrorD2
    ] {
        let transformed = pattern.transform(transform);

        // Convert relative moves to absolute positions
        let mut valid = true;
        let mut moves = Vec::new();

        for (delta_row, delta_col) in &transformed.relative_moves {
            let abs_row = empty_row as i32 + delta_row;
            let abs_col = empty_col as i32 + delta_col;

            // Check bounds
            if abs_row < 0 || abs_col < 0
                || abs_row >= state.size() as i32
                || abs_col >= state.size() as i32 {
                valid = false;
                break;
            }

            moves.push((abs_row as usize, abs_col as usize));
        }

        if valid && check_structure(state, empty_pos, &transformed.structure) {
            return Some(moves);
        }
    }

    None  // Pattern doesn't match at this position
}
```

## Fundamental Patterns (Tile-Agnostic)

### 1. Two-Tile Swap
```
Before:     After:
□ A         A □
```
**Moves**: Right → Left (or any direction pair)
**Cost**: 2
**Invariant**: Same pattern regardless of A's label or direction

### 2. Three-Tile Corner Rotation
```
Before:     After:
□ A         C □
B C    →    B A
```
**Moves**: Right → Down → Left (or any corner orientation)
**Cost**: 3
**Invariant**: Same pattern at all 4 corners of any size grid

### 3. Four-Tile Linear Shift
```
Before:        After:
□ A B C   →    D □ A B
D
```
**Moves**: Right → Right → Right → Down (or any linear direction)
**Cost**: 4
**Invariant**: Same pattern horizontally or vertically

## Implementation Strategy

### Phase 1: Relative Pattern Representation
- Define `RelativePattern` struct with `Vec<(i32, i32)>` deltas
- Add pattern transformation functions (rotate, mirror)
- Update `PatternCatalog` to use relative patterns

### Phase 2: Pattern Matching
- Implement `matches_pattern()` with transformation iteration
- Add structure validation (check tiles form expected shape)
- Return absolute moves if pattern matches

### Phase 3: Solver Integration
- Replace hardcoded pattern attempts with dynamic matching
- For each pattern, try all transformations at current empty position
- Only explore pattern if it structurally matches

## Performance Benefits

**Current system**:
- 4 corner patterns + 2 edge patterns = 6 patterns per node
- Most fail immediately (wrong position)
- Wasted computation

**Tile-agnostic system**:
- 2-3 fundamental patterns per node
- Each tried with 8 transformations = 16-24 attempts
- Only applied if structural match detected
- **Selective**: Far fewer wasted applications

## Example: Detecting "Any Corner Rotation"

```rust
// Current: Must be at exactly (0,0) or (0,2) or (2,0) or (2,2)
if empty_pos == (0, 0) {
    try_pattern(top_left_corner_cw);
}

// Tile-agnostic: Works anywhere tiles form corner shape
let corner_pattern = RelativePattern {
    moves: vec![(0,1), (1,0), (0,-1)],
    structure: CornerTriple,
};

if let Some(moves) = matches_pattern(state, empty_pos, &corner_pattern) {
    // Pattern applies here regardless of position or orientation!
    apply_pattern(moves);
}
```

## Conclusion

The current implementation is **positionally hardcoded** and not tile-agnostic. A proper pattern system would:

1. ✅ Use **relative coordinates** not absolute positions
2. ✅ Recognize patterns through **transformation iteration**
3. ✅ Be **tile-label independent** (only cares about structure)
4. ✅ Reduce pattern count (4 corners → 1 corner pattern)
5. ✅ Enable **selective matching** (only try when structure fits)

This would make patterns both more **correct** (matches fundamental mechanics) and more **performant** (fewer wasted attempts, better selectivity).
