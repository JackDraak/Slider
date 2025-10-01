/// Final benchmark: Hash-table patterns vs all other approaches
///
/// Compares:
/// 1. Standard A* (no patterns)
/// 2. Absolute-position patterns
/// 3. Relative patterns with transformation iteration (slow)
/// 4. Hash-table based patterns (O(1) lookup)

use slider::model::{AStarSolver, Difficulty, EntropyCalculator, ManhattanDistance, PuzzleState};
use slider::controller::ShuffleController;
use std::time::Instant;

fn main() {
    println!("=== Hash-Table Pattern Performance Benchmark ===\n");

    let grid_size = 8;

    for difficulty in [Difficulty::Easy, Difficulty::Medium] {
        println!("Testing {:?} difficulty on {}×{} grid:", difficulty, grid_size, grid_size);
        test_difficulty(difficulty, grid_size);
        println!("\n{}\n", "=".repeat(70));
    }
}

fn test_difficulty(difficulty: Difficulty, grid_size: usize) {
    const TRIALS: usize = 5;

    let mut times_normal: Vec<u128> = Vec::new();
    let mut times_absolute: Vec<u128> = Vec::new();
    let mut times_relative: Vec<u128> = Vec::new();
    let mut times_hash: Vec<u128> = Vec::new();

    for trial in 0..TRIALS {
        let mut puzzle = PuzzleState::new(grid_size).unwrap();
        let shuffler = ShuffleController::new(grid_size).unwrap();
        let calculator = ManhattanDistance;
        shuffler.shuffle(&mut puzzle, difficulty, &calculator);

        let manhattan = calculator.calculate(&puzzle);
        println!("\n  Trial {}: Manhattan = {}", trial + 1, manhattan);

        // Test 1: Standard
        let start = Instant::now();
        let sol_normal = AStarSolver::new().solve_with_path(&puzzle);
        let time_normal = start.elapsed();

        // Test 2: Absolute patterns
        let start = Instant::now();
        let sol_absolute = AStarSolver::with_patterns(grid_size).solve_with_path(&puzzle);
        let time_absolute = start.elapsed();

        // Test 3: Relative patterns (slow iteration)
        let start = Instant::now();
        let sol_relative = AStarSolver::with_relative_patterns().solve_with_path(&puzzle);
        let time_relative = start.elapsed();

        // Test 4: Hash-table patterns (FAST!)
        let start = Instant::now();
        let sol_hash = AStarSolver::with_pattern_hash().solve_with_path(&puzzle);
        let time_hash = start.elapsed();

        match (&sol_normal, &sol_absolute, &sol_relative, &sol_hash) {
            (Some(p_n), Some(p_a), Some(p_r), Some(p_h)) => {
                times_normal.push(time_normal.as_micros());
                times_absolute.push(time_absolute.as_micros());
                times_relative.push(time_relative.as_micros());
                times_hash.push(time_hash.as_micros());

                println!("    Moves: {}", p_n.len());
                println!("    Standard:  {:?}", time_normal);
                println!("    Absolute:  {:?}", time_absolute);
                println!("    Relative:  {:?}", time_relative);
                println!("    Hash:      {:?} ⭐", time_hash);

                assert_eq!(p_n.len(), p_a.len());
                assert_eq!(p_n.len(), p_r.len());
                assert_eq!(p_n.len(), p_h.len());
            }
            _ => {
                println!("    TIMEOUT");
            }
        }
    }

    if !times_normal.is_empty() {
        let avg_normal = times_normal.iter().sum::<u128>() / times_normal.len() as u128;
        let avg_absolute = times_absolute.iter().sum::<u128>() / times_absolute.len() as u128;
        let avg_relative = times_relative.iter().sum::<u128>() / times_relative.len() as u128;
        let avg_hash = times_hash.iter().sum::<u128>() / times_hash.len() as u128;

        println!("\n  === SUMMARY ===");
        println!("  Standard:  {:7}µs  (baseline)", avg_normal);
        println!("  Absolute:  {:7}µs  ({:+.1}%)",
            avg_absolute,
            ((avg_absolute as f64 - avg_normal as f64) / avg_normal as f64) * 100.0
        );
        println!("  Relative:  {:7}µs  ({:+.1}%)",
            avg_relative,
            ((avg_relative as f64 - avg_normal as f64) / avg_normal as f64) * 100.0
        );
        println!("  Hash:      {:7}µs  ({:+.1}%) ⭐",
            avg_hash,
            ((avg_hash as f64 - avg_normal as f64) / avg_normal as f64) * 100.0
        );

        let speedup_vs_relative = avg_relative as f64 / avg_hash as f64;
        println!("\n  Hash vs Relative: {:.2}x faster!", speedup_vs_relative);

        if avg_hash < avg_normal {
            let speedup = avg_normal as f64 / avg_hash as f64;
            println!("  ✓ Hash table is {:.2}x FASTER than baseline!", speedup);
        } else {
            println!("  ✗ Hash table still slower than baseline");
        }
    }
}
