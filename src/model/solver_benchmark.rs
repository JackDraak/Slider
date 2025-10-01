use super::entropy::Difficulty;
use super::ida_star_solver::IDAStarSolver;
use super::puzzle_state::PuzzleState;
use super::solver::AStarSolver;
use crate::controller::ShuffleController;
use crate::model::{EntropyCalculator, ManhattanDistance, PerformanceTimer};

/// Benchmark results for a single solver run
#[derive(Debug, Clone)]
pub struct SolverBenchmarkResult {
    pub solver_name: String,
    pub puzzle_size: usize,
    pub difficulty: Difficulty,
    pub manhattan_distance: u32,
    pub solution_length: u32,
    pub time_micros: u64,
    pub nodes_explored: Option<usize>, // Some solvers track this
}

impl SolverBenchmarkResult {
    pub fn print_summary(&self) {
        println!("\n=== {} ===", self.solver_name);
        println!("  Puzzle: {}×{} {:?}", self.puzzle_size, self.puzzle_size, self.difficulty);
        println!("  Manhattan Distance: {}", self.manhattan_distance);
        println!("  Solution Length: {} moves", self.solution_length);
        println!("  Time: {:.2}ms", self.time_micros as f64 / 1000.0);
        if let Some(nodes) = self.nodes_explored {
            println!("  Nodes Explored: {}", nodes);
            println!("  Nodes/ms: {:.0}", nodes as f64 / (self.time_micros as f64 / 1000.0));
        }
    }
}

/// Compares A* (plain), A* (patterns), and IDA* solvers on the same puzzle
pub fn compare_solvers(puzzle: &PuzzleState) -> (Option<SolverBenchmarkResult>, Option<SolverBenchmarkResult>, Option<SolverBenchmarkResult>) {
    let manhattan_calc = ManhattanDistance;
    let manhattan = manhattan_calc.calculate(puzzle);

    // Benchmark A* WITHOUT patterns
    let astar_plain_result = {
        let timer = PerformanceTimer::start();
        let solver = AStarSolver::new();
        let path = solver.solve_with_path(puzzle);
        let time_micros = timer.elapsed_micros();

        path.map(|p| SolverBenchmarkResult {
            solver_name: "A* (Plain)".to_string(),
            puzzle_size: puzzle.size(),
            difficulty: Difficulty::Medium,
            manhattan_distance: manhattan,
            solution_length: p.len() as u32,
            time_micros,
            nodes_explored: None,
        })
    };

    // Benchmark A* WITH patterns
    let astar_pattern_result = {
        let timer = PerformanceTimer::start();
        let solver = AStarSolver::with_pattern_hash();
        let path = solver.solve_with_path(puzzle);
        let time_micros = timer.elapsed_micros();

        path.map(|p| SolverBenchmarkResult {
            solver_name: "A* (Patterns)".to_string(),
            puzzle_size: puzzle.size(),
            difficulty: Difficulty::Medium,
            manhattan_distance: manhattan,
            solution_length: p.len() as u32,
            time_micros,
            nodes_explored: None,
        })
    };

    // Benchmark IDA*
    let ida_result = {
        let timer = PerformanceTimer::start();
        let mut solver = IDAStarSolver::new();
        let path = solver.solve_with_path(puzzle);
        let time_micros = timer.elapsed_micros();
        let nodes = solver.nodes_explored();

        path.map(|p| SolverBenchmarkResult {
            solver_name: "IDA* (New)".to_string(),
            puzzle_size: puzzle.size(),
            difficulty: Difficulty::Medium,
            manhattan_distance: manhattan,
            solution_length: p.len() as u32,
            time_micros,
            nodes_explored: Some(nodes),
        })
    };

    (astar_plain_result, astar_pattern_result, ida_result)
}

/// Runs a comprehensive benchmark comparing all solvers
pub fn run_comprehensive_benchmark(grid_size: usize, difficulty: Difficulty, iterations: usize) {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║          SOLVER COMPARISON BENCHMARK                         ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!("\nConfiguration:");
    println!("  Grid Size: {}×{}", grid_size, grid_size);
    println!("  Difficulty: {:?}", difficulty);
    println!("  Iterations: {}", iterations);

    let mut astar_plain_times = Vec::new();
    let mut astar_pattern_times = Vec::new();
    let mut ida_times = Vec::new();

    for i in 0..iterations {
        println!("\n--- Iteration {}/{} ---", i + 1, iterations);

        // Create a fresh puzzle
        let mut puzzle = PuzzleState::new(grid_size).expect("valid size");
        let shuffle_controller = ShuffleController::new(grid_size).expect("valid size");
        let manhattan_calc = ManhattanDistance;

        shuffle_controller.shuffle(&mut puzzle, difficulty, &manhattan_calc);

        // Compare solvers
        let (astar_plain, astar_pattern, ida_result) = compare_solvers(&puzzle);

        if let Some(ref result) = astar_plain {
            result.print_summary();
            astar_plain_times.push(result.time_micros);
        }

        if let Some(ref result) = astar_pattern {
            result.print_summary();
            astar_pattern_times.push(result.time_micros);
        }

        if let Some(ref result) = ida_result {
            result.print_summary();
            ida_times.push(result.time_micros);
        }

        // Print comparison
        if let (Some(plain), Some(pattern)) = (&astar_plain, &astar_pattern) {
            let speedup = plain.time_micros as f64 / pattern.time_micros as f64;
            println!("\n  🚀 Pattern Speedup: {:.2}x faster!", speedup);
        }
    }

    // Print aggregate statistics
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                   AGGREGATE RESULTS                          ║");
    println!("╚══════════════════════════════════════════════════════════════╝");

    if !astar_plain_times.is_empty() {
        let plain_avg = astar_plain_times.iter().sum::<u64>() as f64 / astar_plain_times.len() as f64;
        println!("\nA* (Plain - No Patterns):");
        println!("  Average Time: {:.2}ms", plain_avg / 1000.0);
        println!("  Min Time: {:.2}ms", *astar_plain_times.iter().min().unwrap() as f64 / 1000.0);
        println!("  Max Time: {:.2}ms", *astar_plain_times.iter().max().unwrap() as f64 / 1000.0);
    }

    if !astar_pattern_times.is_empty() {
        let pattern_avg = astar_pattern_times.iter().sum::<u64>() as f64 / astar_pattern_times.len() as f64;
        println!("\nA* (With Patterns):");
        println!("  Average Time: {:.2}ms", pattern_avg / 1000.0);
        println!("  Min Time: {:.2}ms", *astar_pattern_times.iter().min().unwrap() as f64 / 1000.0);
        println!("  Max Time: {:.2}ms", *astar_pattern_times.iter().max().unwrap() as f64 / 1000.0);
    }

    if !ida_times.is_empty() {
        let ida_avg = ida_times.iter().sum::<u64>() as f64 / ida_times.len() as f64;
        println!("\nIDA*:");
        println!("  Average Time: {:.2}ms", ida_avg / 1000.0);
        println!("  Min Time: {:.2}ms", *ida_times.iter().min().unwrap() as f64 / 1000.0);
        println!("  Max Time: {:.2}ms", *ida_times.iter().max().unwrap() as f64 / 1000.0);
    }

    if !astar_plain_times.is_empty() && !astar_pattern_times.is_empty() {
        let plain_avg = astar_plain_times.iter().sum::<u64>() as f64 / astar_plain_times.len() as f64;
        let pattern_avg = astar_pattern_times.iter().sum::<u64>() as f64 / astar_pattern_times.len() as f64;
        let speedup = plain_avg / pattern_avg;

        println!("\n🎯 PATTERN DATABASE IMPACT:");
        println!("   {:.2}x FASTER with patterns!", speedup);
        println!("   That's a {:.0}% improvement!", (speedup - 1.0) * 100.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_3x3() {
        let mut puzzle = PuzzleState::new(3).unwrap();
        puzzle.apply_immediate_move((2, 1));
        puzzle.apply_immediate_move((2, 0));

        let (astar_plain, astar_pattern, ida) = compare_solvers(&puzzle);

        assert!(astar_plain.is_some());
        assert!(astar_pattern.is_some());
        assert!(ida.is_some());

        // All should find the same solution length
        let plain_len = astar_plain.unwrap().solution_length;
        let pattern_len = astar_pattern.unwrap().solution_length;
        let ida_len = ida.unwrap().solution_length;
        assert_eq!(plain_len, pattern_len);
        assert_eq!(plain_len, ida_len);
    }
}
