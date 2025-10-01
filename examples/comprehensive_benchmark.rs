//! Comprehensive benchmarking tool for solver performance analysis
//! 
//! This tool runs extensive benchmarks to:
//! 1. Measure solver failure rates across difficulty levels
//! 2. Correlate shuffle move count with solver performance
//! 3. Identify patterns in failed cases
//! 4. Provide baseline metrics for optimization comparison

use slider::{
    controller::game_controller::GameController,
    model::{Difficulty, AStarSolver},
};
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone)]
struct BenchmarkResult {
    shuffle_moves: usize,
    manhattan_distance: u32,
    solve_time_ms: Option<f64>,
    solution_length: Option<u32>,
    nodes_explored: Option<usize>,
    failed: bool,
}

#[derive(Debug)]
struct DifficultyStats {
    total_samples: usize,
    failures: usize,
    avg_shuffle_moves: f64,
    avg_manhattan: f64,
    avg_solve_time_ms: f64,
    avg_solution_length: f64,
    failure_rate: f64,
    worst_case_time_ms: f64,
    worst_case_moves: usize,
}

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║          COMPREHENSIVE SOLVER BENCHMARK                     ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    let grid_size = 4;
    let samples_per_difficulty = 100;
    
    println!("Configuration:");
    println!("  Grid Size: {}×{}", grid_size, grid_size);
    println!("  Samples per difficulty: {}", samples_per_difficulty);
    println!("  Solver: A* (Shortest Path Heuristic)");
    println!();

    // Benchmark each difficulty level
    let difficulties = [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard];
    let mut all_results = HashMap::new();

    for difficulty in &difficulties {
        println!("--- Benchmarking {:?} ---", difficulty);
        let results = benchmark_difficulty(grid_size, *difficulty, samples_per_difficulty);
        let stats = analyze_results(&results);
        
        println!("  Samples: {}", stats.total_samples);
        println!("  Failures: {} ({:.1}%)", stats.failures, stats.failure_rate * 100.0);
        println!("  Avg shuffle moves: {:.1}", stats.avg_shuffle_moves);
        println!("  Avg Manhattan distance: {:.1}", stats.avg_manhattan);
        if stats.avg_solve_time_ms > 0.0 {
            println!("  Avg solve time: {:.2}ms", stats.avg_solve_time_ms);
            println!("  Avg solution length: {:.1}", stats.avg_solution_length);
            println!("  Worst case time: {:.2}ms", stats.worst_case_time_ms);
            println!("  Worst case moves: {}", stats.worst_case_moves);
        }
        println!();

        all_results.insert(*difficulty, (results, stats));
    }

    // Analyze failure patterns
    analyze_failure_patterns(&all_results);

    // Correlate shuffle moves with performance
    analyze_move_correlation(&all_results);

    // Generate recommendations
    generate_recommendations(&all_results);
}

fn benchmark_difficulty(grid_size: usize, difficulty: Difficulty, samples: usize) -> Vec<BenchmarkResult> {
    let mut results = Vec::with_capacity(samples);
    let solver = AStarSolver::new();

    for i in 0..samples {
        if (i + 1) % 20 == 0 {
            print!("  Progress: {}/{}\r", i + 1, samples);
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }

        let mut controller = GameController::new(grid_size).unwrap();
        controller.new_game(difficulty);

        let shuffle_result = controller.last_shuffle_result().unwrap();
        let manhattan = controller.current_entropy();

        // Try to solve
        let start_time = Instant::now();
        let solve_result = solver.solve_with_path(controller.state());
        let solve_time = start_time.elapsed();

        match solve_result {
            Some(solution) => {
                results.push(BenchmarkResult {
                    shuffle_moves: shuffle_result.moves_made,
                    manhattan_distance: manhattan,
                    solve_time_ms: Some(solve_time.as_secs_f64() * 1000.0),
                    solution_length: Some(solution.len() as u32),
                    nodes_explored: None, // Would need to modify solver to expose this
                    failed: false,
                });
            }
            None => {
                results.push(BenchmarkResult {
                    shuffle_moves: shuffle_result.moves_made,
                    manhattan_distance: manhattan,
                    solve_time_ms: None,
                    solution_length: None,
                    nodes_explored: None,
                    failed: true,
                });
            }
        }
    }

    println!("  Progress: {}/{}", samples, samples);
    results
}

fn analyze_results(results: &[BenchmarkResult]) -> DifficultyStats {
    let total_samples = results.len();
    let failures = results.iter().filter(|r| r.failed).count();
    
    let successful: Vec<_> = results.iter().filter(|r| !r.failed).collect();
    
    let avg_shuffle_moves = results.iter().map(|r| r.shuffle_moves as f64).sum::<f64>() / total_samples as f64;
    let avg_manhattan = results.iter().map(|r| r.manhattan_distance as f64).sum::<f64>() / total_samples as f64;
    
    let (avg_solve_time_ms, avg_solution_length, worst_case_time_ms, worst_case_moves) = if !successful.is_empty() {
        let avg_solve_time = successful.iter()
            .map(|r| r.solve_time_ms.unwrap())
            .sum::<f64>() / successful.len() as f64;
        let avg_solution = successful.iter()
            .map(|r| r.solution_length.unwrap() as f64)
            .sum::<f64>() / successful.len() as f64;
        let worst_time = successful.iter()
            .map(|r| r.solve_time_ms.unwrap())
            .fold(0.0f64, f64::max);
        let worst_moves = successful.iter()
            .map(|r| r.shuffle_moves)
            .max()
            .unwrap_or(0);
        
        (avg_solve_time, avg_solution, worst_time, worst_moves)
    } else {
        (0.0, 0.0, 0.0, 0)
    };

    DifficultyStats {
        total_samples,
        failures,
        avg_shuffle_moves,
        avg_manhattan,
        avg_solve_time_ms,
        avg_solution_length,
        failure_rate: failures as f64 / total_samples as f64,
        worst_case_time_ms,
        worst_case_moves,
    }
}

fn analyze_failure_patterns(all_results: &HashMap<Difficulty, (Vec<BenchmarkResult>, DifficultyStats)>) {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                    FAILURE PATTERN ANALYSIS                 ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    for (difficulty, (results, _stats)) in all_results {
        let failures: Vec<_> = results.iter().filter(|r| r.failed).collect();
        
        if failures.is_empty() {
            println!("✓ {:?}: No failures detected", difficulty);
            continue;
        }

        println!("✗ {:?} - {} failures:", difficulty, failures.len());
        
        let avg_shuffle_moves = failures.iter().map(|r| r.shuffle_moves as f64).sum::<f64>() / failures.len() as f64;
        let avg_manhattan = failures.iter().map(|r| r.manhattan_distance as f64).sum::<f64>() / failures.len() as f64;
        let max_shuffle_moves = failures.iter().map(|r| r.shuffle_moves).max().unwrap();
        let max_manhattan = failures.iter().map(|r| r.manhattan_distance).max().unwrap();

        println!("  Average shuffle moves: {:.1}", avg_shuffle_moves);
        println!("  Average Manhattan distance: {:.1}", avg_manhattan);
        println!("  Maximum shuffle moves: {}", max_shuffle_moves);
        println!("  Maximum Manhattan distance: {}", max_manhattan);
        
        // Look for patterns
        let high_move_failures = failures.iter().filter(|r| r.shuffle_moves > 50).count();
        let high_manhattan_failures = failures.iter().filter(|r| r.manhattan_distance > 40).count();
        
        if high_move_failures > 0 {
            println!("  {} failures had >50 shuffle moves", high_move_failures);
        }
        if high_manhattan_failures > 0 {
            println!("  {} failures had >40 Manhattan distance", high_manhattan_failures);
        }
        println!();
    }
}

fn analyze_move_correlation(all_results: &HashMap<Difficulty, (Vec<BenchmarkResult>, DifficultyStats)>) {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║               SHUFFLE MOVES CORRELATION ANALYSIS            ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    for (difficulty, (results, _stats)) in all_results {
        let successful: Vec<_> = results.iter().filter(|r| !r.failed).collect();
        
        if successful.is_empty() {
            continue;
        }

        // Correlate shuffle moves with solve time
        let moves_vs_time: Vec<(usize, f64)> = successful.iter()
            .map(|r| (r.shuffle_moves, r.solve_time_ms.unwrap()))
            .collect();
        
        // Simple correlation analysis
        let avg_moves = successful.iter().map(|r| r.shuffle_moves as f64).sum::<f64>() / successful.len() as f64;
        let avg_time = successful.iter().map(|r| r.solve_time_ms.unwrap()).sum::<f64>() / successful.len() as f64;
        
        let numerator: f64 = moves_vs_time.iter()
            .map(|(moves, time)| (*moves as f64 - avg_moves) * (time - avg_time))
            .sum();
        
        let moves_variance: f64 = successful.iter()
            .map(|r| (r.shuffle_moves as f64 - avg_moves).powi(2))
            .sum();
        
        let time_variance: f64 = successful.iter()
            .map(|r| (r.solve_time_ms.unwrap() - avg_time).powi(2))
            .sum();
        
        let correlation = if moves_variance > 0.0 && time_variance > 0.0 {
            numerator / (moves_variance * time_variance).sqrt()
        } else {
            0.0
        };

        println!("{:?}:", difficulty);
        println!("  Correlation (shuffle moves vs solve time): {:.3}", correlation);
        
        // Find problematic ranges
        let mut buckets = HashMap::new();
        for result in &successful {
            let bucket = (result.shuffle_moves / 10) * 10; // 10-move buckets
            let entry = buckets.entry(bucket).or_insert((Vec::new(), 0.0, 0));
            entry.0.push(result.solve_time_ms.unwrap());
            entry.1 += result.solve_time_ms.unwrap();
            entry.2 += 1;
        }

        println!("  Performance by shuffle move ranges:");
        let mut buckets: Vec<_> = buckets.into_iter().collect();
        buckets.sort_by_key(|(bucket, _)| *bucket);
        
        for (bucket, (times, total, count)) in buckets {
            let avg_time = total / count as f64;
            let max_time = times.iter().fold(0.0f64, |a, &b| a.max(b));
            println!("    {}-{} moves: {:.2}ms avg, {:.2}ms max", bucket, bucket + 9, avg_time, max_time);
        }
        println!();
    }
}

fn generate_recommendations(all_results: &HashMap<Difficulty, (Vec<BenchmarkResult>, DifficultyStats)>) {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                      RECOMMENDATIONS                         ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    let has_failures = all_results.values().any(|(_, stats)| stats.failures > 0);
    
    if has_failures {
        println!("🚨 CRITICAL ISSUES FOUND:");
        println!("  • Solver failures detected - optimization needed");
        println!("  • Consider implementing better heuristics");
        println!("  • Empty cell path abstraction may help");
        println!();
    }

    // Check for performance degradation
    for (difficulty, (results, stats)) in all_results {
        if stats.failure_rate > 0.1 {
            println!("⚠️  {:?} has {:.1}% failure rate - HIGH PRIORITY", difficulty, stats.failure_rate * 100.0);
        } else if stats.failure_rate > 0.0 {
            println!("⚠️  {:?} has {:.1}% failure rate - monitor", difficulty, stats.failure_rate * 100.0);
        }
        
        if stats.worst_case_time_ms > 1000.0 {
            println!("⚠️  {:?} worst case: {:.1}ms - consider optimization", difficulty, stats.worst_case_time_ms);
        }
        
        if stats.avg_shuffle_moves > 80.0 {
            println!("ℹ️  {:?} avg shuffle moves: {:.1} - may be over-shuffling", difficulty, stats.avg_shuffle_moves);
        }
    }

    println!();
    println!("🎯 OPTIMIZATION PRIORITIES:");
    
    if has_failures {
        println!("  1. FIX FAILURES - Implement better heuristics (Walking Distance, Empty Cell Path)");
        println!("  2. Add diagnostic logging to understand failure patterns");
        println!("  3. Consider iterative deepening for hard cases");
    } else {
        println!("  1. PERFORMANCE - Optimize solve times for worst cases");
        println!("  2. PRECOMPUTATION - Cache common patterns");
        println!("  3. PARALLELIZATION - Use rayon for large searches");
    }

    println!();
    println!("📊 NEXT STEPS:");
    println!("  1. Implement empty cell path heuristic");
    println!("  2. Add comprehensive logging to solver");
    println!("  3. Create A/B testing framework");
    println!("  4. Test geometric decomposition approach");
}
