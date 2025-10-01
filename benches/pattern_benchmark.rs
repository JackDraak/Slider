/// Benchmark comparing A* solver with and without pattern optimization
///
/// This measures the performance difference when using pattern-based move
/// exploration vs standard immediate-move-only exploration.

use slider::model::{AStarSolver, Difficulty, PuzzleState};
use std::time::Instant;

fn main() {
    println!("=== Pattern-Based A* Optimization Benchmark ===\n");

    // Test on multiple difficulty levels
    for difficulty in [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard] {
        println!("Testing {:?} difficulty:", difficulty);
        benchmark_difficulty(difficulty);
        println!();
    }
}

fn benchmark_difficulty(difficulty: Difficulty) {
    const TRIALS: usize = 10;
    let grid_size = 4;

    let mut times_without_patterns: Vec<u128> = Vec::new();
    let mut times_with_patterns: Vec<u128> = Vec::new();

    for trial in 0..TRIALS {
        // Create and shuffle puzzle
        let mut puzzle = PuzzleState::new(grid_size).unwrap();

        // Shuffle to desired difficulty
        use slider::controller::ShuffleController;
        use slider::model::ManhattanDistance;
        let mut shuffler = ShuffleController::new(grid_size).unwrap();
        let calculator = ManhattanDistance;
        shuffler.shuffle(&mut puzzle, difficulty, &calculator);

        // Test WITHOUT patterns
        let solver_normal = AStarSolver::new();
        let start = Instant::now();
        let solution_normal = solver_normal.solve_with_path(&puzzle);
        let time_normal = start.elapsed();

        // Test WITH patterns
        let solver_patterns = AStarSolver::with_patterns(grid_size);
        let start = Instant::now();
        let solution_patterns = solver_patterns.solve_with_path(&puzzle);
        let time_patterns = start.elapsed();

        // Verify both found solutions
        if let (Some(path_normal), Some(path_patterns)) = (&solution_normal, &solution_patterns) {
            times_without_patterns.push(time_normal.as_micros());
            times_with_patterns.push(time_patterns.as_micros());

            // Both should find optimal solutions
            assert_eq!(path_normal.len(), path_patterns.len(),
                "Pattern solver found different solution length!");

            println!("  Trial {}: {} moves | Normal: {:?} | Patterns: {:?}",
                trial + 1,
                path_normal.len(),
                time_normal,
                time_patterns
            );
        } else {
            println!("  Trial {}: TIMEOUT (skipping)", trial + 1);
        }
    }

    // Calculate statistics
    if !times_without_patterns.is_empty() {
        let avg_without = times_without_patterns.iter().sum::<u128>() / times_without_patterns.len() as u128;
        let avg_with = times_with_patterns.iter().sum::<u128>() / times_with_patterns.len() as u128;

        let speedup = avg_without as f64 / avg_with as f64;
        let percent_change = ((avg_with as f64 - avg_without as f64) / avg_without as f64) * 100.0;

        println!("\n  Average time without patterns: {}µs", avg_without);
        println!("  Average time with patterns:    {}µs", avg_with);
        println!("  Speedup: {:.2}x", speedup);
        println!("  Performance change: {:.1}%", percent_change);
    }
}
