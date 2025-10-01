/// Benchmark comparing tile-agnostic relative patterns vs absolute patterns
///
/// This measures the performance difference between:
/// 1. Standard A* (no patterns)
/// 2. Absolute-position patterns (old implementation)
/// 3. Relative tile-agnostic patterns (new implementation)

use slider::model::{AStarSolver, Difficulty, EntropyCalculator, ManhattanDistance, PuzzleState};
use slider::controller::ShuffleController;
use std::time::Instant;

fn main() {
    println!("=== Relative Pattern Performance Comparison ===\n");

    // Test on 8×8 puzzles where patterns show more benefit
    let grid_size = 8;

    for difficulty in [Difficulty::Easy, Difficulty::Medium] {
        println!("Testing {:?} difficulty on {}×{} grid:", difficulty, grid_size, grid_size);
        test_difficulty(difficulty, grid_size);
        println!("\n{}\n", "=".repeat(60));
    }
}

fn test_difficulty(difficulty: Difficulty, grid_size: usize) {
    const TRIALS: usize = 5;

    let mut times_normal: Vec<u128> = Vec::new();
    let mut times_absolute: Vec<u128> = Vec::new();
    let mut times_relative: Vec<u128> = Vec::new();

    for trial in 0..TRIALS {
        // Create and shuffle puzzle
        let mut puzzle = PuzzleState::new(grid_size).unwrap();
        let shuffler = ShuffleController::new(grid_size).unwrap();
        let calculator = ManhattanDistance;
        shuffler.shuffle(&mut puzzle, difficulty, &calculator);

        let manhattan = calculator.calculate(&puzzle);
        println!("\n  Trial {}: Manhattan distance = {}", trial + 1, manhattan);

        // Test 1: Standard solver (no patterns)
        let solver_normal = AStarSolver::new();
        let start = Instant::now();
        let solution_normal = solver_normal.solve_with_path(&puzzle);
        let time_normal = start.elapsed();

        // Test 2: Absolute-position patterns
        let solver_absolute = AStarSolver::with_patterns(grid_size);
        let start = Instant::now();
        let solution_absolute = solver_absolute.solve_with_path(&puzzle);
        let time_absolute = start.elapsed();

        // Test 3: Relative tile-agnostic patterns
        let solver_relative = AStarSolver::with_relative_patterns();
        let start = Instant::now();
        let solution_relative = solver_relative.solve_with_path(&puzzle);
        let time_relative = start.elapsed();

        // Record results
        match (&solution_normal, &solution_absolute, &solution_relative) {
            (Some(path_n), Some(path_a), Some(path_r)) => {
                times_normal.push(time_normal.as_micros());
                times_absolute.push(time_absolute.as_micros());
                times_relative.push(time_relative.as_micros());

                println!("    Solution: {} moves", path_n.len());
                println!("    Standard:  {:?}", time_normal);
                println!("    Absolute:  {:?}", time_absolute);
                println!("    Relative:  {:?}", time_relative);

                // Verify all found optimal solutions
                assert_eq!(path_n.len(), path_a.len(), "Absolute patterns found suboptimal solution");
                assert_eq!(path_n.len(), path_r.len(), "Relative patterns found suboptimal solution");
            }
            _ => {
                println!("    TIMEOUT (skipping)");
            }
        }
    }

    // Calculate statistics
    if !times_normal.is_empty() {
        let avg_normal = times_normal.iter().sum::<u128>() / times_normal.len() as u128;
        let avg_absolute = times_absolute.iter().sum::<u128>() / times_absolute.len() as u128;
        let avg_relative = times_relative.iter().sum::<u128>() / times_relative.len() as u128;

        println!("\n  === SUMMARY ===");
        println!("  Average Standard:  {:7}µs  (baseline)", avg_normal);
        println!("  Average Absolute:  {:7}µs  ({:+.1}%)",
            avg_absolute,
            ((avg_absolute as f64 - avg_normal as f64) / avg_normal as f64) * 100.0
        );
        println!("  Average Relative:  {:7}µs  ({:+.1}%)",
            avg_relative,
            ((avg_relative as f64 - avg_normal as f64) / avg_normal as f64) * 100.0
        );

        // Compare absolute vs relative
        let speedup = avg_absolute as f64 / avg_relative as f64;
        println!("\n  Relative vs Absolute: {:.2}x speedup", speedup);

        if speedup > 1.0 {
            println!("  ✓ Tile-agnostic patterns are FASTER");
        } else {
            println!("  ✗ Tile-agnostic patterns still slower");
        }
    }
}
