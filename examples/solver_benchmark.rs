/// Benchmark tool for comparing A* and IDA* solvers
///
/// Usage:
///   cargo run --example solver_benchmark -- [size] [difficulty] [iterations]
///
/// Examples:
///   cargo run --example solver_benchmark -- 4 medium 5
///   cargo run --example solver_benchmark -- 5 medium 3

use slider::model::{Difficulty, run_comprehensive_benchmark};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let grid_size = if args.len() > 1 {
        args[1].parse().unwrap_or(4)
    } else {
        4
    };

    let difficulty = if args.len() > 2 {
        match args[2].to_lowercase().as_str() {
            "easy" => Difficulty::Easy,
            "hard" => Difficulty::Hard,
            _ => Difficulty::Medium,
        }
    } else {
        Difficulty::Medium
    };

    let iterations = if args.len() > 3 {
        args[3].parse().unwrap_or(5)
    } else {
        5
    };

    run_comprehensive_benchmark(grid_size, difficulty, iterations);
}
