/// Test pattern optimization on larger puzzles (8×8)
///
/// This tests the hypothesis that patterns help more on larger search spaces.

use slider::model::{AStarSolver, Difficulty, EntropyCalculator, ManhattanDistance, PuzzleState};
use slider::controller::ShuffleController;
use std::time::Instant;

fn main() {
    println!("=== Large Puzzle (8×8) Pattern Test ===\n");

    for difficulty in [Difficulty::Easy, Difficulty::Medium] {
        test_difficulty(difficulty);
        println!("\n{}\n", "=".repeat(50));
    }
}

fn test_difficulty(difficulty: Difficulty) {
    println!("Testing {:?} difficulty:\n", difficulty);

    let grid_size = 8;

    // Create puzzle at specified difficulty
    let mut puzzle = PuzzleState::new(grid_size).unwrap();
    let shuffler = ShuffleController::new(grid_size).unwrap();
    let calculator = ManhattanDistance;
    shuffler.shuffle(&mut puzzle, difficulty, &calculator);

    let manhattan = calculator.calculate(&puzzle);
    println!("Puzzle shuffled with Manhattan distance: {}", manhattan);
    println!("Starting solve attempts...\n");

    // Test standard solver
    println!("Testing STANDARD solver:");
    let solver_normal = AStarSolver::new();
    let start = Instant::now();
    let solution_normal = solver_normal.solve_with_path(&puzzle);
    let time_normal = start.elapsed();

    match &solution_normal {
        Some(path) => {
            println!("  ✓ Found solution: {} moves", path.len());
            println!("  ✓ Time: {:?}", time_normal);
        }
        None => {
            println!("  ✗ TIMEOUT after {:?}", time_normal);
        }
    }

    println!();

    // Test pattern solver
    println!("Testing PATTERN solver:");
    let solver_patterns = AStarSolver::with_patterns(grid_size);
    let start = Instant::now();
    let solution_patterns = solver_patterns.solve_with_path(&puzzle);
    let time_patterns = start.elapsed();

    match &solution_patterns {
        Some(path) => {
            println!("  ✓ Found solution: {} moves", path.len());
            println!("  ✓ Time: {:?}", time_patterns);
        }
        None => {
            println!("  ✗ TIMEOUT after {:?}", time_patterns);
        }
    }

    println!();

    // Compare results
    match (solution_normal, solution_patterns) {
        (Some(path_normal), Some(path_patterns)) => {
            let speedup = time_normal.as_secs_f64() / time_patterns.as_secs_f64();
            println!("=== COMPARISON ===");
            println!("Solution lengths: {} vs {}", path_normal.len(), path_patterns.len());
            println!("Speedup: {:.2}x", speedup);
            if speedup > 1.0 {
                println!("✓ Patterns are FASTER on 8×8!");
            } else {
                println!("✗ Patterns still slower on 8×8");
            }
        }
        (Some(_), None) => {
            println!("Standard solved, patterns timed out");
        }
        (None, Some(_)) => {
            println!("✓ Patterns solved, standard timed out — patterns help!");
        }
        (None, None) => {
            println!("Both timed out — puzzle too complex");
        }
    }
}
