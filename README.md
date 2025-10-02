# Slider - Sliding Tile Puzzle Game

A Rust implementation of the classic sliding-tile puzzle (also known as the "fifteen puzzle"). This implementation features entropy-based difficulty levels, guaranteed solvable shuffles, and a clean Model-Controller-Presenter architecture.

![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)

## Features

- **Configurable Grid Sizes**: Play on grids from 3×3 to 15×15 (default 4×4)
- **Smart Move System**:
  - Immediate moves for adjacent tiles
  - Chain moves by clicking any tile in line with the empty cell
  - **Smooth animations**: 200ms tile slide with ease-out cubic interpolation
- **Auto-Solve with A\* Pathfinding**:
  - Watch optimal solution play out with smooth animations
  - Recalculates on each invocation (handles stop/resume with manual moves)
  - 700ms interval between moves (200ms animation + 500ms pause)
- **Triple Entropy Metrics**: Compare three different complexity measurements:
  - Manhattan Distance (fast heuristic)
  - Shortest Path Heuristic with linear conflict penalties
  - A\* Actual Solution Length (exact optimal path, never times out on 4×4)
- **Entropy-Based Difficulty**: Three difficulty levels (Easy/Medium/Hard) based on puzzle disorder
- **Guaranteed Solvable**: Shuffles use mechanical simulation to ensure all puzzles are solvable
- **Visual Feedback**:
  - Color-coded tiles (green=correct, gray=incorrect, yellow=hover)
  - Smooth sliding animations for professional feel
- **Performance Metrics**: Toggle to see A\* calculation time for algorithmic insight
- **Real-Time Stats**: Move counter and entropy display
- **Debug Logging**: Console output showing auto-solve behavior (solution paths, move tracking)

## Quick Start

### Prerequisites

- Rust 1.70 or later
- Cargo

### Installation

```bash
git clone https://github.com/yourusername/slider.git
cd slider
cargo build --release
```

### Running the Game

```bash
# Run with default 4×4 grid
cargo run --release

# Run with custom grid size (e.g., 5×5)
cargo run --release 5
```

### Controls

- **Click a tile**: Move it to the empty space (if legal)
- **New Game**: Start a new shuffled puzzle with selected difficulty
- **Reset**: Return to solved state
- **Auto Solve**: Watch the computer solve the puzzle optimally (1 move per second)
- **Stop Solve**: Pause the auto-solver mid-solution
- **Difficulty buttons**: Choose Easy, Medium, or Hard before starting a new game
- **Show Performance**: Toggle to display A\* solver calculation time

## How It Works

### The Puzzle Mechanic

The puzzle consists of a grid of numbered tiles with one empty cell. Only tiles adjacent to the empty cell can slide into it. The goal is to arrange all tiles in sequential order.

**Legal Moves**:
- **Immediate moves**: Adjacent tiles (up/down/left/right from empty cell)
- **Chain moves**: Tiles in line with empty cell automatically shift in sequence

**Example**: In a 4×4 puzzle with the empty cell at position (3,3), clicking tile at (3,0) will shift tiles (3,2) → (3,3), then (3,1) → (3,2), then (3,0) → (3,1).

### Entropy & Difficulty

Entropy measures the "disorder" of the puzzle - how far tiles are from their home positions. The game displays three different metrics:

1. **Manhattan Distance** (Fast Heuristic)
   - Sum of horizontal + vertical distances for all tiles from their home positions
   - Underestimates the true solution length but calculates instantly
   - Example: A tile 2 rows and 3 columns away contributes 5 to the total

2. **Shortest Path Heuristic** (Improved Estimate)
   - Manhattan Distance plus linear conflict penalties
   - Detects when tiles block each other in the same row/column
   - More accurate than Manhattan, still calculates in microseconds

3. **A\* Actual Solution Length** (Exact Optimal Path)
   - Uses A\* pathfinding to compute the true minimum number of moves needed
   - This is the **real** puzzle difficulty - no estimation
   - Computationally expensive (milliseconds to seconds for complex puzzles)
   - Only calculated for 4×4 puzzles with Manhattan distance < 50

**Performance Toggle**: Enable "Show Performance" to see how long the A\* solver takes. The fast heuristics (Manhattan and Shortest Path) compute so quickly their timing is not displayed, while the A\* solver's calculation time gives insight into algorithmic complexity.

Difficulty levels target different entropy thresholds:
- **Easy**: Low entropy (fewer moves required)
- **Medium**: Moderate entropy (10-20 optimal moves typical)
- **Hard**: High entropy (30+ optimal moves)

### Solvability Guarantee

All shuffles are guaranteed solvable because they're generated using the same move mechanics as gameplay, with no backtracking. This ensures every puzzle state is reachable from the solved state.

## Architecture

The codebase follows the **Model-Controller-Presenter (MCP)** pattern with clean separation of concerns:

### Model Layer (`src/model/`)
Core game logic and data structures, completely independent of UI:

- **`puzzle_state.rs`**: Core game state, grid management, and move operations
- **`tile.rs`**: Tile abstraction with content enum (Numeric/Image placeholder)
- **`move_validator.rs`**: Legal move validation and chain move resolution
- **`entropy.rs`**: Multiple entropy calculators (Manhattan Distance, Shortest Path, Enhanced)
- **`solver.rs`**: A\* pathfinding with memory-efficient implementation and cancellation support
- **`enhanced_heuristic.rs`**: Advanced heuristic combining multiple complexity metrics
- **`error.rs`**: Comprehensive error types (no panics in library code)
- **`performance.rs`**: High-precision timing utilities for algorithm benchmarking

### Controller Layer (`src/controller/`)
Game orchestration and business logic:

- **`game_controller.rs`**: Complete game orchestration, move handling, auto-solve, metric caching
- **`shuffle_controller.rs`**: Entropy-based puzzle shuffling with solvability guarantees

### Presenter Layer (`src/presenter/`)
UI rendering and user interaction:

- **`gui_presenter.rs`**: egui-based GUI with comprehensive controls and real-time state display
- **`tile_renderer.rs`**: Visual tile rendering with smooth animations and hover effects

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_chain_move_horizontal
```

**Test Coverage**: 59 unit tests covering move validation, entropy calculations, A\* solver, error handling, shuffle mechanics, and game controller logic.

### Building Documentation

```bash
# Generate and open documentation
cargo doc --open
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy
```

## Programmatic Usage

### Basic Game Control

```rust
use slider::{GameController, Difficulty};

// Create a new 4×4 puzzle (returns Result for proper error handling)
let mut game = GameController::new(4).expect("valid grid size");

// Start a new game with medium difficulty
game.new_game(Difficulty::Medium);

// Make a move by clicking a tile position
let success = game.handle_click((3, 2));

// Check game state
if game.is_solved() {
    println!("Solved in {} moves!", game.move_count());
    println!("Final entropy: {}", game.current_entropy());
}

// Get detailed entropy metrics (cached for performance)
let metrics = game.all_entropy_metrics();
println!("Manhattan: {}", metrics.manhattan_distance);
println!("Heuristic: {}", metrics.shortest_path_heuristic);
println!("Actual optimal: {}", metrics.actual_solution_length);
println!("A* calc time: {}μs", metrics.performance.actual_time_micros);

// Auto-solve the puzzle
game.start_auto_solve();
```

### Direct Puzzle State Manipulation

```rust
use slider::model::{PuzzleState, ManhattanDistance, AStarSolver};

// Create and manipulate puzzle state directly
let mut puzzle = PuzzleState::new(4)?;

// Apply moves programmatically
puzzle.apply_immediate_move((3, 2))?;
puzzle.apply_immediate_move((2, 2))?;

// Calculate entropy
let entropy = ManhattanDistance.calculate(&puzzle);
println!("Current entropy: {}", entropy);

// Solve the puzzle
let solver = AStarSolver::new();
if let Some(solution_length) = solver.solve(&puzzle) {
    println!("Optimal solution: {} moves", solution_length);
    
    // Get full solution path
    if let Some(path) = solver.solve_with_path(&puzzle) {
        println!("Solution path: {:?}", path);
    }
}
```

### Custom Entropy Calculation

```rust
use slider::model::{PuzzleState, EnhancedHeuristic, ShortestPathHeuristic};

let puzzle = PuzzleState::new(4)?;

// Compare different heuristics
let enhanced = EnhancedHeuristic;
let shortest_path = ShortestPathHeuristic;

let enhanced_score = enhanced.calculate(&puzzle);
let shortest_score = shortest_path.calculate(&puzzle);

println!("Enhanced heuristic: {}", enhanced_score);
println!("Shortest path heuristic: {}", shortest_score);
```

### Benchmarking Solver Performance

```rust
use slider::model::{PuzzleState, AStarSolver, PerformanceTimer};
use std::time::Duration;

let solver = AStarSolver::new();
let puzzle = PuzzleState::new(4)?;

// Time the solver
let timer = PerformanceTimer::new();
let solution = solver.solve(&puzzle);
let elapsed = timer.elapsed();

println!("Solver took: {:?}", elapsed);
println!("Solution length: {:?}", solution);
```

## Performance Optimizations

Recent optimizations have made the A\* solver production-ready:

1. **Memory-Efficient A\***: Replaced exponential memory growth (Box parent chains) with Vec-indexed storage
   - Before: O(2^n) memory usage causing OOM on complex puzzles
   - After: O(n) memory usage, can solve significantly larger puzzles

2. **Fast State Hashing**: Replaced string concatenation with u64 hashing
   - Before: String allocation per state (~100-1000ns per hash)
   - After: Direct integer hashing (~10ns per hash)
   - Result: 10-100x speedup in A\* pathfinding

3. **Improved Heap Ordering**: Added tie-breaking for equal f-scores
   - Prioritizes nodes closer to goal when f-scores are equal
   - Prevents arbitrary exploration order
   - Increased iteration limit to 2M (handles all 4×4 puzzles)

4. **Metric Caching**: Entropy calculations cached per puzzle state
   - Prevents redundant computation during frame updates
   - Only recalculates when puzzle state changes

5. **Animation System**: Smooth 200ms tile slides with ease-out cubic
   - Professional polish without performance impact
   - Integrates seamlessly with auto-solve playback

These optimizations enable the A\* solver to find optimal solutions for all 4×4 puzzles without timeout.

## Future Enhancements

- **Image-based puzzles**: Replace numbers with custom images (abstraction already in place)
- **Undo/Redo**: Move history tracking is implemented, just needs UI
- **Solver improvements**: Parallel A\* search using rayon for larger grids
- **Timer**: Track solve time for speedrun mode
- **Leaderboards**: Local high scores per difficulty level
- **Keyboard controls**: Arrow keys or WASD for tile movement
- **Animation speed control**: User-adjustable animation duration
- **Sound effects**: Audio feedback for moves and completion

## Contributing

Contributions are welcome! Please follow the existing architecture:

1. **TDD Approach**: Write tests first (Uncle Bob's red-green method)
2. **MCP Pattern**: Keep Model, Controller, and Presenter concerns separated
3. **Documentation**: Add doc comments for all public APIs
4. **Testing**: Maintain test coverage for new features

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

- Inspired by the classic 15-puzzle invented by Noyes Palmer Chapman in 1874
- Built with [egui](https://github.com/emilk/egui) - an immediate mode GUI library
- Architecture follows clean code principles from Robert C. Martin's work
