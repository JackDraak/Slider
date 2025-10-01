//! Benchmark comparing Empty Cell Path heuristic with original Shortest Path heuristic
//! 
//! This example demonstrates the improvement in solver performance and reliability
//! when using the new Empty Cell Path heuristic for complex puzzle configurations.

use std::time::{Duration, Instant};
use std::collections::HashMap;
use slider::{
    PuzzleState, Difficulty, ShuffleController
};
use slider::model::{AStarSolver, AStarSolverEmptyCell, ManhattanDistance};

#[derive(Debug, Clone)]
struct BenchmarkResult {
    solver_name: String,
    success_rate: f64,
    avg_solve_time: Duration,
    avg_solution_length: f64,
    worst_case_time: Duration,
    total_failures: usize,
    samples: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║          EMPTY CELL PATH HEURISTIC BENCHMARK                ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    
    let grid_size = 4;
    let samples_per_difficulty = 100;
    
    println!("\nConfiguration:");
    println!("  Grid Size: {}×{}", grid_size, grid_size);
    println!("  Samples per difficulty: {}", samples_per_difficulty);
    println!("  Solvers: A* (Shortest Path) vs A* (Empty Cell Path)");
    
    // Test both solvers on different difficulties
    let difficulties = [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard];
    let mut all_results = HashMap::new();
    
    for &difficulty in &difficulties {
        println!("\n--- Benchmarking {:?} ---", difficulty);
        
        // Test original solver
        let original_result = benchmark_solver(
            "A* (Shortest Path)",
            grid_size,
            difficulty,
            samples_per_difficulty,
            |state| {
                let solver = AStarSolver::new();
                solver.solve_with_path(state)
            }
        );
        
        // Test new Empty Cell Path solver
        let empty_cell_result = benchmark_solver(
            "A* (Empty Cell Path)",
            grid_size,
            difficulty,
            samples_per_difficulty,
            |state| {
                let solver = AStarSolverEmptyCell::new();
                solver.solve_with_path(state)
            }
        );
        
        all_results.insert(difficulty, (original_result.clone(), empty_cell_result.clone()));
        
        // Print immediate results
        print_solver_results(&original_result);
        print_solver_results(&empty_cell_result);
        
        // Show improvement
        println!("\n📊 IMPROVEMENT ANALYSIS:");
        let success_improvement = empty_cell_result.success_rate - original_result.success_rate;
        let time_improvement = original_result.avg_solve_time.as_secs_f64() - empty_cell_result.avg_solve_time.as_secs_f64();
        let failure_reduction = original_result.total_failures - empty_cell_result.total_failures;
        
        if success_improvement > 0.0 {
            println!("  ✅ Success rate improved by {:.1}%", success_improvement * 100.0);
        }
        if time_improvement > 0.0 {
            println!("  ⚡ Average solve time improved by {:.2}ms", time_improvement * 1000.0);
        }
        if failure_reduction > 0 {
            println!("  🎯 Failures reduced by {}", failure_reduction);
        }
        if success_improvement == 0.0 && time_improvement == 0.0 && failure_reduction == 0 {
            println!("  ➖ No significant difference detected");
        }
    }
    
    // Print comprehensive comparison
    print_comprehensive_comparison(&all_results);
    
    // Print recommendations
    print_recommendations(&all_results);
    
    Ok(())
}

fn benchmark_solver<F>(
    solver_name: &str,
    grid_size: usize,
    difficulty: Difficulty,
    samples: usize,
    solve_fn: F,
) -> BenchmarkResult
where
    F: Fn(&PuzzleState) -> Option<Vec<(usize, usize)>>,
{
    let mut total_time = Duration::ZERO;
    let mut successful_solves = 0;
    let mut total_solution_length = 0u64;
    let mut worst_case_time = Duration::ZERO;
    let mut total_failures = 0;
    
    let mut rng = rand::thread_rng();
    
    for i in 0..samples {
        // Generate shuffled puzzle
        let mut puzzle = PuzzleState::new(grid_size).unwrap();
        let controller = ShuffleController::new(grid_size).unwrap();
        let calculator = ManhattanDistance;
        
        let shuffle_result = controller.shuffle_with_result(&mut puzzle, difficulty, &calculator);
        
        let shuffle_moves = shuffle_result.moves_made;
        
        // Benchmark solve time
        let start_time = Instant::now();
        let solution = solve_fn(&puzzle);
        let solve_time = start_time.elapsed();
        
        match solution {
            Some(path) => {
                successful_solves += 1;
                total_solution_length += path.len() as u64;
                total_time += solve_time;
                worst_case_time = worst_case_time.max(solve_time);
                
                // Verify solution correctness
                if !verify_solution(&puzzle, &path) {
                    eprintln!("WARNING: Invalid solution found for {} at sample {}", solver_name, i);
                }
            }
            None => {
                total_failures += 1;
                // Still count the time for failed attempts
                total_time += solve_time;
                worst_case_time = worst_case_time.max(solve_time);
            }
        }
    }
    
    let success_rate = successful_solves as f64 / samples as f64;
    let avg_solve_time = if successful_solves > 0 {
        total_time / successful_solves as u32
    } else {
        Duration::ZERO
    };
    let avg_solution_length = if successful_solves > 0 {
        total_solution_length as f64 / successful_solves as f64
    } else {
        0.0
    };
    
    BenchmarkResult {
        solver_name: solver_name.to_string(),
        success_rate,
        avg_solve_time,
        avg_solution_length,
        worst_case_time,
        total_failures,
        samples,
    }
}

fn verify_solution(initial_state: &PuzzleState, solution: &[(usize, usize)]) -> bool {
    let mut test_state = initial_state.clone();
    
    for &move_pos in solution {
        if !test_state.apply_immediate_move(move_pos) {
            return false;
        }
    }
    
    test_state.is_solved()
}

fn print_solver_results(result: &BenchmarkResult) {
    println!("  {}: {:.1}% success rate", result.solver_name, result.success_rate * 100.0);
    println!("    Average solve time: {:.2}ms", result.avg_solve_time.as_secs_f64() * 1000.0);
    println!("    Average solution length: {:.1} moves", result.avg_solution_length);
    println!("    Worst case time: {:.2}ms", result.worst_case_time.as_secs_f64() * 1000.0);
    println!("    Failures: {}/{}", result.total_failures, result.samples);
}

fn print_comprehensive_comparison(
    results: &HashMap<Difficulty, (BenchmarkResult, BenchmarkResult)>
) {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                    COMPREHENSIVE COMPARISON                ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    
    println!("\n┌─────────────┬─────────────────────┬─────────────────────┬─────────────┐");
    println!("│ Difficulty  │ A* (Shortest Path) │ A* (Empty Cell)    │ Improvement │");
    println!("├─────────────┼─────────────────────┼─────────────────────┼─────────────┤");
    
    for difficulty in [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard] {
        if let Some((original, empty_cell)) = results.get(&difficulty) {
            let success_improvement = empty_cell.success_rate - original.success_rate;
            let time_improvement = original.avg_solve_time.as_secs_f64() - empty_cell.avg_solve_time.as_secs_f64();
            
            println!("│ {:11} │ {:6.1}% ({:4.1}ms) │ {:6.1}% ({:4.1}ms) │ {:8.1}% │",
                format!("{:?}", difficulty),
                original.success_rate * 100.0,
                original.avg_solve_time.as_secs_f64() * 1000.0,
                empty_cell.success_rate * 100.0,
                empty_cell.avg_solve_time.as_secs_f64() * 1000.0,
                success_improvement * 100.0
            );
            
            if time_improvement > 0.0 {
                println!("│             │                     │                     │ {:7.1}ms faster │", time_improvement * 1000.0);
            }
        }
    }
    
    println!("└─────────────┴─────────────────────┴─────────────────────┴─────────────┘");
    
    println!("\nFailure Rate Comparison:");
    println!("┌─────────────┬─────────────────────┬─────────────────────┬─────────────┐");
    println!("│ Difficulty  │ A* (Shortest Path) │ A* (Empty Cell)    │ Reduction   │");
    println!("├─────────────┼─────────────────────┼─────────────────────┼─────────────┤");
    
    for difficulty in [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard] {
        if let Some((original, empty_cell)) = results.get(&difficulty) {
            let original_failure_rate = original.total_failures as f64 / original.samples as f64 * 100.0;
            let empty_cell_failure_rate = empty_cell.total_failures as f64 / empty_cell.samples as f64 * 100.0;
            let reduction = original_failure_rate - empty_cell_failure_rate;
            
            println!("│ {:11} │ {}/{} ({:4.1}%)    │ {}/{} ({:4.1}%)    │ {:7.1}%    │",
                format!("{:?}", difficulty),
                original.total_failures, original.samples, original_failure_rate,
                empty_cell.total_failures, empty_cell.samples, empty_cell_failure_rate,
                reduction
            );
        }
    }
    
    println!("└─────────────┴─────────────────────┴─────────────────────┴─────────────┘");
}

fn print_recommendations(
    results: &HashMap<Difficulty, (BenchmarkResult, BenchmarkResult)>
) {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                      RECOMMENDATIONS                         ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    
    let mut hard_improvement = false;
    let mut medium_improvement = false;
    let mut easy_improvement = false;
    
    if let Some((original, empty_cell)) = results.get(&Difficulty::Hard) {
        if empty_cell.success_rate > original.success_rate {
            println!("🎯 MAJOR IMPROVEMENT: Empty Cell Path reduces Hard puzzle failures");
            hard_improvement = true;
        }
        if empty_cell.total_failures < original.total_failures {
            println!("✅ CRITICAL: {} fewer failures on Hard puzzles", 
                original.total_failures - empty_cell.total_failures);
        }
    }
    
    if let Some((original, empty_cell)) = results.get(&Difficulty::Medium) {
        if empty_cell.avg_solve_time < original.avg_solve_time {
            println!("⚡ PERFORMANCE: Empty Cell Path faster on Medium puzzles");
            medium_improvement = true;
        }
    }
    
    if let Some((original, empty_cell)) = results.get(&Difficulty::Easy) {
        if empty_cell.success_rate == original.success_rate && 
           empty_cell.total_failures == original.total_failures {
            println!("✓ CONSISTENCY: Both solvers perform equally on Easy puzzles");
            easy_improvement = true;
        }
    }
    
    if hard_improvement {
        println!("\n🚀 RECOMMENDATION: Deploy Empty Cell Path heuristic immediately");
        println!("   It addresses the critical failure rate on complex puzzles");
        println!("   and provides a solid foundation for further optimizations.");
    } else {
        println!("\n⚠️  RECOMMENDATION: Further heuristic refinement needed");
        println!("   The Empty Cell Path heuristic needs additional tuning");
        println!("   to effectively handle complex puzzle configurations.");
    }
    
    println!("\n📈 NEXT STEPS:");
    println!("   1. If Hard failures are eliminated: Add pattern databases");
    println!("   2. If still failing: Implement IDA* fallback mechanism");
    println!("   3. Consider geometric decomposition for remaining edge cases");
    println!("   4. Add comprehensive logging to understand failure patterns");
}
