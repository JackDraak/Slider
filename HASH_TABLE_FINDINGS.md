# Hash-Table Pattern Matching - Performance Analysis

## Goal
Replace O(patterns × 8) transformation iteration with O(1) hash lookup for pattern matching.

## Implementation
Created `PatternHashTable` that:
1. Precomputes all pattern transformations
2. Maps local tile configuration (bitmask) to matching patterns
3. Attempts O(1) lookup instead of iterating transformations

## Performance Results

### 8×8 Easy Difficulty
- **Standard**: 69.8ms (baseline)
- **Absolute patterns**: 69.4ms (-0.6%)
- **Relative patterns**: 106.4ms (+52%)
- **Hash table**: 289.1ms (+314%) ⚠️

### 8×8 Medium Difficulty
- **Standard**: ~4s (baseline)
- **Hash table**: ~16s (+300%) ⚠️

**Conclusion**: Hash-table approach is **3-4x SLOWER** than even the slow relative pattern iteration!

## Root Cause Analysis

### The Fundamental Problem

The hash table requires **subset matching**, not exact matching:
- A pattern that needs "tile to the right" should match ANY configuration that has a tile to the right
- This means pattern with bitmask `0001` should match configs `0001`, `0011`, `0101`, `0111`, `1001`, etc.

### What Went Wrong

Current implementation:
```rust
for (required_config, candidates) in &self.lookup {
    if (config.adjacent_mask & required_config.adjacent_mask) == required_config.adjacent_mask {
        // Check this pattern
    }
}
```

This is **O(n) iteration through all hash table entries**, not O(1) lookup!

### Why Hash Tables Don't Work Here

**Problem**: Need to find all entries where `required_bits ⊆ current_bits`

**Hash table assumption**: Exact key matching in O(1)

**Reality**:
- Can't hash on exact config (need subset matching)
- Generating all supersets as keys = O(2^8) = 256 entries per pattern
- Still need to try all matching patterns anyway

**Actual complexity**: O(patterns × transformations) = same as before, just with more overhead!

## Alternative Approaches That Might Work

### 1. Bit-Trie Structure
```rust
struct BitTrie {
    // Each bit position is a branch
    // Leaf nodes contain matching patterns
}
```
- Insert patterns at nodes where required bits match
- Query: traverse trie following set bits
- Complexity: O(log n) instead of O(n)
- **Might be worth trying**

### 2. Precomputed All-Configs Table
```rust
// For 8 bits = 256 possible configs
let lookup: [Vec<Pattern>; 256] = precompute_all();
```
- Array index = bitmask value
- O(1) lookup, but 256 entries × patterns stored
- Memory: ~2KB for 2 patterns
- **Fast but memory intensive**

### 3. SIMD Bit Matching
```rust
// Use SIMD to check multiple patterns in parallel
let matches = simd_check_all_patterns(config);
```
- Check 4-8 patterns simultaneously
- Complexity still O(n) but with constant factor improvement
- **Hardware-dependent**

### 4. Give Up on Caching
The simplest solution: **accept that O(patterns × 8) is acceptable**
- Only 2 patterns × 8 transformations = 16 checks per node
- Each check is just bounds checking + tile existence
- **Probably the right answer for this problem size**

## Lessons Learned

1. **Hash tables aren't always the answer**: They work for exact matching, not subset queries
2. **Algorithmic complexity vs real performance**: O(16) simple checks can beat O(1) complex lookup
3. **Premature optimization**: The relative patterns were "slow" at 50% overhead, but hash table made it 300% slower!
4. **Pattern count matters**: With only 2-3 patterns, sophisticated data structures add more overhead than they save

## Recommendation

**Keep the simple relative pattern iteration** (`with_relative_patterns()`):
- Correctness: ✅ Tile-agnostic, rotation-invariant
- Performance: ❌ 50% slower than baseline
- Complexity: Simple and maintainable

**Or revert to no patterns** (`new()`):
- Simplest and fastest for 4×4 and 8×8 puzzles
- Patterns only help on 10×10+ where search depth dominates

## Production Status

- `AStarSolver::new()`: **Recommended** - fastest, simplest
- `AStarSolver::with_patterns(size)`: Absolute patterns - slight overhead
- `AStarSolver::with_relative_patterns()`: Tile-agnostic but 50% slower
- `AStarSolver::with_pattern_hash()`: ⚠️ **Do not use** - 300% slower

The pattern system serves as excellent **educational code** demonstrating:
- Tile-agnostic pattern matching
- Transformation iteration
- Hash table limitations
- Performance profiling methodology

But for production use on puzzle sizes ≤8×8: **disable patterns entirely**.
