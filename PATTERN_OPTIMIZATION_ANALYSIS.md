# Pattern-Based A* Optimization Analysis

## Overview

This document analyzes the implementation and performance characteristics of pattern-based move exploration for the sliding puzzle A* solver.

## Implementation

### Pattern Catalog

Defined in `src/model/pattern_catalog.rs`, the catalog contains common multi-move sequences:

1. **Corner Rotations** (3 moves each):
   - Top-left clockwise: right → down → left
   - Top-right clockwise: down → left → up
   - Bottom-right clockwise: left → up → right
   - Bottom-left clockwise: up → right → down

2. **Edge Shifts** (2 moves each):
   - Top row shift right
   - Left column shift down

### Search Integration

The pattern-based solver (`AStarSolver::with_patterns()`) explores successors in two phases:

1. **Standard immediate moves** (2-4 options based on empty cell position)
2. **Pattern-based moves** (6-10 patterns from catalog)

When a pattern is successfully applied, the resulting state is added to the open set with:
- `g_score` increased by pattern cost (number of moves in pattern)
- Full move sequence stored for path reconstruction

### Path Reconstruction

`SearchNode` now stores `moves_from_parent: Vec<Position>` instead of single move, allowing:
- Single immediate moves: `vec![pos]`
- Pattern sequences: `vec![pos1, pos2, pos3, ...]`

## Performance Benchmark Results

### Test Setup
- **Platform**: 4×4 puzzles with Easy/Medium/Hard difficulties
- **Trials**: 10 per difficulty level
- **Comparison**: Standard A* vs Pattern-based A*

### Results

| Difficulty | Avg Time (Standard) | Avg Time (Patterns) | Speedup | Change |
|-----------|--------------------:|--------------------:|--------:|-------:|
| Easy      | 17µs                | 21µs                | 0.81x   | +23.5% |
| Medium    | 73µs                | 94µs                | 0.78x   | +28.8% |
| Hard      | 327ms               | 383ms               | 0.86x   | +16.9% |

**Conclusion**: Pattern-based optimization is **17-29% slower** across all difficulty levels.

## Analysis: Why Patterns Are Slower

### 1. Increased Branching Factor

**Standard solver**:
- Corner position: 2 immediate moves
- Edge position: 3 immediate moves
- Surrounded: 4 immediate moves
- Average branching factor: ~3

**Pattern solver**:
- Same immediate moves: 2-4
- Plus pattern attempts: 6-10
- Average branching factor: **~10** (3x increase!)

**Impact**: Explores far more states per iteration, overwhelming any depth reduction benefits.

### 2. Pattern Application Overhead

For each pattern:
1. Clone puzzle state
2. Apply 2-3 moves sequentially
3. Check if resulting state is valid/useful
4. Most patterns lead to states that aren't on the optimal path

**Wasted work**: ~60-80% of pattern attempts don't help reach the goal.

### 3. Patterns Not Selective Enough

Current implementation tries **all patterns at every node**, regardless of context.

**Better approach would be**:
- Only try patterns when preconditions match
- Example: "Corner rotation" only when empty cell is in corner AND a specific tile configuration exists
- This requires more sophisticated pattern matching

### 4. 4×4 Puzzles Don't Benefit from Depth Reduction

**Why depth reduction doesn't help**:
- Easy puzzles: 8 moves → exploring 8 levels is already fast
- Medium puzzles: 16-20 moves → patterns don't reduce depth enough to matter
- Hard puzzles: 40-50 moves → search is already highly pruned by heuristic

**Where patterns would help**:
- Much larger puzzles (8×8, 10×10) with solution depths of 200+ moves
- Problems where heuristic is weak (patterns provide better goal-direction)

## Correctness Verification

### Tests Passing

```rust
#[test]
fn test_pattern_solver_finds_optimal_solution() {
    // Verifies pattern solver finds same optimal length as standard solver
    assert_eq!(solution_normal, solution_patterns);
}

#[test]
fn test_pattern_solver_path_correctness() {
    // Verifies returned path actually solves the puzzle
    // (all moves valid, leads to solved state)
}
```

**Result**: ✅ Pattern solver is **correct** — finds optimal solutions, just slower than standard approach.

## Recommendations

### For Current Codebase

**Keep patterns disabled by default** (already done):
```rust
impl AStarSolver {
    pub fn new() -> Self {
        // use_patterns: false
    }
}
```

### To Make Patterns Faster (Future Work)

1. **Add Precondition Checking**:
   ```rust
   pub struct MovePattern {
       precondition: Box<dyn Fn(&PuzzleState) -> bool>,
       // Only try pattern if precondition returns true
   }
   ```

2. **Reduce Pattern Count**:
   - Start with 2-3 most valuable patterns
   - Profile to identify which patterns actually help

3. **Pattern Database Approach**:
   - Precompute costs for solving subproblems (e.g., "get top row solved")
   - Use as enhanced heuristic rather than exploring pattern moves

4. **Larger Puzzles**:
   - Test on 8×8 or larger where depth reduction matters more
   - Patterns more likely to show speedup on deeper searches

### Alternative: Enhanced Heuristic

Instead of exploring pattern moves, use patterns to **improve the heuristic**:

```rust
impl EntropyCalculator for PatternDatabaseHeuristic {
    fn calculate(&self, state: &PuzzleState) -> u32 {
        let base = manhattan_distance(state);
        let pattern_bonus = recognize_problematic_patterns(state);
        base + pattern_bonus
    }
}
```

This gives benefits of pattern recognition without increased branching factor.

## Conclusion

The pattern-based optimization is:
- ✅ **Correctly implemented** — finds optimal solutions
- ❌ **Not performance beneficial** — 17-29% slower on 4×4 puzzles
- 📊 **Educational value** — demonstrates tradeoffs in search algorithms
- 🔬 **Useful for research** — framework exists for future experimentation

The implementation serves as a **research foundation** for exploring more sophisticated pattern databases, but should remain **disabled by default** for production use.

## References

- Korf, R. E. (1997). "Finding Optimal Solutions to Rubik's Cube Using Pattern Databases"
- Culberson, J. & Schaeffer, J. (1998). "Pattern Databases"
- Korf, R. E. & Felner, A. (2002). "Disjoint Pattern Database Heuristics"
