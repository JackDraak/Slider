# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Slider** is a Rust implementation of the sliding-tile puzzle (also known as "fifteen"). The puzzle models entropy in a closed system: tiles are shuffled to introduce entropy (measured as work required to solve), then the player manipulates tiles to return the puzzle to its solved state (zero entropy).

### Puzzle Mechanics

- Grid sizes: 3×3 to 15×15 (default 4×4)
- A 4×4 puzzle has 16 cells with 15 numbered tiles and 1 empty cell
- **Core mechanic**: Tiles adjacent to the empty cell can slide into the empty space
- **Chain moves**: Series of tiles in line with empty cell can be moved sequentially (e.g., click tile 13 when empty is at 16 → moves 15→16, 14→15, 13→14)
- **Legal moves depend on empty cell position**:
  - Corner: 2 immediate moves (+ 4 possible chain destinations)
  - Edge: 3 immediate moves (+ 3 possible chain destinations)
  - Surrounded: 4 immediate moves (+ 2 possible chain destinations)

### Design Principles

1. **Solvability guarantee**: Shuffles use the same move mechanics as gameplay, with no backtracking, ensuring all puzzles are solvable
2. **Entropy measurement**: Two implementations for comparison:
   - Manhattan Distance (sum of distances from home positions)
   - Shortest path length calculation
3. **Difficulty levels**: Entropy thresholds determine Easy/Medium/Hard
4. **Future-proof**: Tile content abstracted for eventual image support

## Architecture

The codebase follows **Model-Controller-Presenter (MCP)** pattern:

### Model Layer (`src/model/`)
- **`puzzle_state.rs`** - Core game state: grid representation, tile positions, empty cell location
- **`tile.rs`** - Tile definition with `TileContent` enum (`Numeric(u32)` | `Image(ImageData)`)
- **`entropy.rs`** - Entropy calculators: Manhattan Distance and Shortest Path Heuristic (MD + linear conflicts)
- **`enhanced_heuristic.rs`** - Enhanced heuristic with corner and edge penalties for A* solver
- **`solver.rs`** - A* pathfinding solver using Enhanced Heuristic
- **`move_validator.rs`** - Validates legal moves based on empty cell position (implements truth table logic)

### Controller Layer (`src/controller/`)
- **`game_controller.rs`** - Orchestrates game logic, processes player input, resolves chain moves into buffered immediate moves
- **`shuffle_controller.rs`** - Board shuffling using immediate moves only, no backtracking, with entropy threshold enforcement
- **`move_history.rs`** - Tracks move count for scoring

### Presenter Layer (`src/presenter/`)
- **`gui_presenter.rs`** - egui-based rendering and input handling
- **`tile_renderer.rs`** - Visual representation abstraction

## Key Implementation Concepts

### Move Resolution
- Player clicks are resolved to determine if tile is in a legal position
- **Immediate moves**: Adjacent tiles move directly into empty space
- **Chain moves**: Non-adjacent tiles in line with empty space trigger buffered sequence
  - Example: Empty at cell 16, click tile at cell 13 → buffer [15→16, 14→15, 13→14]
  - Execute as series of immediate moves with visual feedback

### Shuffle Algorithm
- Uses only immediate moves (no chain move complexity during shuffle)
- Tracks previous empty position to prevent backtracking
- Continues until entropy threshold met for selected difficulty
- Approximately 50+ moves typical for adequate entropy

### Entropy Calculation & A* Solver
Three heuristic methods displayed for analysis:
1. **Manhattan Distance**: `Σ(|current_x - home_x| + |current_y - home_y|)` for all tiles (base heuristic)
2. **Shortest Path Heuristic**: Manhattan Distance + linear conflict penalties (×2)
3. **Enhanced Heuristic** (used by solver): Shortest Path + corner penalties (+3 each) + edge penalties (+2 each)

The A* solver uses the **Enhanced Heuristic** which provides ~60% improvement over Manhattan Distance on 5×5 puzzles. The GUI displays all three metrics plus actual solution length in real-time. Results are **cached** and only recalculated when state changes. Solutions are cached for instant replay via auto-solve. For performance, actual solution is only calculated for smaller puzzles (4×4 with MD < 50).

### Solvability Invariant
Shuffles constructed using legal moves guarantee solvability. The puzzle state maintains an invariant that all reachable states are solvable because they're generated through the same mechanics used for solving.

## Development Commands

```bash
# Build project
cargo build

# Run application
cargo run

# Run tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocaptures

# Check without building
cargo check

# Format code
cargo fmt

# Lint
cargo clippy
```

## Testing Strategy

Follow Uncle Bob's red-green TDD methodology:
1. Write failing test for new behavior
2. Implement minimal code to pass
3. Refactor while keeping tests green

### Critical Test Coverage
- Move validation for all empty cell positions (corner/edge/surrounded)
- Shuffle solvability verification
- Entropy calculation correctness (all three methods)
- A\* solver correctness and optimality
- Chain move resolution accuracy
- Boundary conditions (min/max grid sizes)
- Error handling (invalid sizes, etc.)
- **Current test count**: 59 passing tests

## GUI Framework

Currently using **egui** (immediate mode GUI). Architecture supports future refactoring to alternative frameworks (iced, druid, tauri) through Presenter layer abstraction.

### Input Handling
- Mouse click detection for tile selection
- Keyboard support (future: arrow keys, WASD)
- Visual feedback during chain move execution

## Module Dependencies

```
main.rs → gui_presenter → game_controller → puzzle_state
                                          → move_validator
                                          → shuffle_controller → entropy
                                                              → move_validator
                                          → solver (A* pathfinding)
                                          → error (Result types)
```

The Model layer has no dependencies on Controller or Presenter, maintaining clean separation of concerns.

## Performance & Optimization Notes

### A\* Solver Optimizations (Completed)

1. **Enhanced Heuristic** (Current)
   - Base: Manhattan Distance + linear conflicts (×2)
   - Added: Corner tile penalties (+3 each)
   - Added: Last row/column penalties (+2 each when multiple wrong)
   - **Impact**: ~60% improvement over Manhattan Distance, enables solving 5×5 Medium in <50ms

2. **Memory Efficiency**
   - Uses `parent_index: Option<usize>` with Vec-based node storage
   - **Impact**: O(n) memory usage instead of exponential growth

3. **State Hashing**
   - u64 hash using `DefaultHasher` (no string allocation)
   - **Impact**: 10-100x speedup over string concatenation

4. **Solution Caching**
   - Solved paths cached for instant replay
   - Background threading prevents UI freezing
   - Entropy metrics cached per state version

### Error Handling (Completed)

- All constructors return `Result<T, PuzzleError>` instead of panicking
- Error types: `PuzzleError`, `SolverError`, `AutoSolveError`
- Proper error propagation through all layers
- Library-grade API (no unexpected panics)