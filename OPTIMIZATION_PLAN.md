# Solver Optimization Plan

## Problem Statement

The current A* solver fails on some 4×4 hard puzzles (and occasionally medium) due to:
1. High variability in shuffle difficulty within the same difficulty category
2. Manhattan distance heuristic underestimating actual solution length by 60-70%
3. No visibility into shuffle complexity (move count not exposed)
4. Missing geometric abstractions that could provide better heuristics

## Key Insights & Abstractions

### 1. Empty Cell Path Abstraction
Instead of thinking about "which tile to move," think about "where should the empty cell go?"

**Key insight**: The empty cell's journey through the grid creates a deterministic wake. Each position it visits changes the local environment and affects system entropy.

**Potential optimizations**:
- Precompute "optimal empty cell paths" for common patterns
- Use empty cell position as primary heuristic component
- Track empty cell's "entropy contribution" at each position

### 2. Entropy Flow Analysis
Think of entropy as flowing through the system rather than being a static property.

**Key insight**: Moving the empty cell can temporarily increase local entropy while decreasing global entropy - this is the "temporary increase" phenomenon you mentioned.

**Potential optimizations**:
- Identify "entropy bottlenecks" where temporary increases are necessary
- Precompute "entropy flow patterns" for common configurations
- Use entropy gradient instead of absolute entropy

### 3. Geometric Decomposition
Break the puzzle into geometric subproblems that can be solved independently.

**Key insight**: Corners, edges, and center have different mechanical constraints.

**Potential optimizations**:
- Solve corners first (most constrained)
- Use different heuristics for different regions
- Exploit symmetries and invariants

## Implementation Plan

### Phase 1: Diagnostics & Visibility (Immediate)

1. **Expose Shuffle Move Count**
   - Modify `ShuffleController` to track and report actual moves made
   - Add to GUI display
   - Correlate move count with solver performance

2. **Comprehensive Benchmarking**
   - Test 100+ shuffles per difficulty level
   - Record failure rates and solve times
   - Identify patterns in failed cases

3. **Solver Performance Profiling**
   - Track nodes explored vs solution depth
   - Identify where heuristic fails most
   - Measure "heuristic accuracy" (estimated vs actual)

### Phase 2: Empty Cell Path Optimization (Medium Effort)

1. **Empty Cell Centric Heuristic**
   ```rust
   struct EmptyCellHeuristic {
       // Precomputed optimal paths from each position to each target
       position_values: [[u32; GRID_SIZE]; GRID_SIZE],
       // Entropy contribution map for each empty cell position
       entropy_map: [[u32; GRID_SIZE]; GRID_SIZE],
   }
   ```

2. **Path-Based State Representation**
   - Store empty cell's "journey history" as part of state
   - Use this to predict future entropy changes
   - Identify when temporary increases are beneficial

3. **Wake Pattern Recognition**
   - Precompute common empty cell movement patterns
   - Use pattern matching to predict optimal next moves
   - Cache frequently used wake patterns

### Phase 3: Geometric Decomposition (High Effort)

1. **Region-Based Solving**
   ```rust
   struct GeometricSolver {
       corner_solver: CornerSolver,
       edge_solver: EdgeSolver, 
       center_solver: CenterSolver,
   }
   ```

2. **Symmetry Exploitation**
   - Rotate/reflect puzzles to canonical forms
   - Solve in canonical space, transform back
   - Cache solutions for symmetric patterns

3. **Entropy Flow Optimization**
   - Precompute "entropy gradients" for each position
   - Use gradient descent on empty cell movement
   - Identify and exploit "entropy highways"

### Phase 4: Hybrid Approach (Research)

1. **Multi-Heuristic A***
   - Combine Manhattan, empty-cell, and entropy-flow heuristics
   - Use heuristic switching based on puzzle state
   - Learn which heuristic works best for which patterns

2. **Machine Learning Enhancement**
   - Train on successful solving patterns
   - Predict optimal next moves based on state
   - Use neural networks for pattern recognition

## A-B Testing Framework

### Baseline Measurement
- Current A* solver performance
- Failure rate by difficulty
- Average solve time
- Memory usage

### New Solver Variants
1. **Empty Cell Heuristic Only**
2. **Empty Cell + Manhattan Combination**
3. **Geometric Decomposition**
4. **Entropy Flow Optimization**
5. **Hybrid Multi-Heuristic**

### Success Metrics
- Reduce failure rate on 4×4 hard puzzles from current ~X% to <5%
- Maintain or improve average solve time for easy/medium
- Memory usage stays within current bounds
- Solution optimality preserved (no longer paths)

## Implementation Strategy

### Side-by-Side Development
1. Keep current solver unchanged as baseline
2. Implement new solvers as separate modules
3. Create unified benchmarking interface
4. Allow runtime solver selection for comparison

### Incremental Deployment
1. Phase 1: Diagnostics only (no solver changes)
2. Phase 2: Add empty cell heuristic as option
3. Phase 3: Add geometric decomposition
4. Phase 4: Hybrid approaches

### Rollback Plan
- All new solvers are opt-in
- Baseline solver remains default
- Easy to disable any optimization that hurts performance

## Expected Outcomes

### Conservative Estimates
- 50% reduction in hard puzzle failure rate
- 10-20% improvement in average solve time
- Better understanding of puzzle complexity

### Optimistic Estimates
- 90% reduction in hard puzzle failure rate
- 2-3x improvement on hardest puzzles
- New insights into puzzle mechanics

## Risk Assessment

### Low Risk
- Phase 1 diagnostics (no solver changes)
- Empty cell heuristic addition

### Medium Risk  
- Geometric decomposition complexity
- Performance regression on easy puzzles

### High Risk
- Machine learning approaches
- Major architectural changes

## Timeline

### Week 1: Phase 1 Complete
- Shuffle move counting
- Comprehensive benchmarking
- Performance profiling

### Week 2: Phase 2 Implementation
- Empty cell heuristic
- Path-based state representation
- Initial testing

### Week 3: Phase 3 Implementation
- Geometric decomposition
- Symmetry exploitation
- Integration testing

### Week 4: Phase 4 Research
- Hybrid approaches
- Final optimization
- Production deployment

This plan addresses the real performance issues while exploring the geometric and mechanical abstractions that could lead to breakthrough improvements.
