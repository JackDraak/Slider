/// Quick test of Walking Distance heuristic quality
///
/// Usage: cargo run --example test_walking_distance

use slider::controller::ShuffleController;
use slider::model::{Difficulty, EntropyCalculator, ManhattanDistance, PuzzleState, ShortestPathHeuristic, WalkingDistance};

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║     WALKING DISTANCE HEURISTIC QUALITY TEST                  ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    for grid_size in [3, 4, 5] {
        test_grid_size(grid_size);
    }
}

fn test_grid_size(grid_size: usize) {
    println!("\n━━━ {}×{} Puzzle ━━━", grid_size, grid_size);

    let manhattan = ManhattanDistance;
    let shortest_path = ShortestPathHeuristic;
    let walking_dist = WalkingDistance::new(grid_size);

    let shuffle_controller = ShuffleController::new(grid_size).unwrap();

    for difficulty in [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard] {
        println!("\n{:?} Difficulty:", difficulty);

        let mut puzzle = PuzzleState::new(grid_size).unwrap();
        shuffle_controller.shuffle(&mut puzzle, difficulty, &manhattan);

        let man_score = manhattan.calculate(&puzzle);
        let sp_score = shortest_path.calculate(&puzzle);
        let wd_score = walking_dist.calculate(&puzzle);

        println!("  Manhattan Distance:      {}", man_score);
        println!("  Shortest Path Heuristic: {}", sp_score);
        println!("  Walking Distance:        {}", wd_score);

        let improvement_sp = (sp_score as f64 / man_score as f64 - 1.0) * 100.0;
        let improvement_wd = (wd_score as f64 / man_score as f64 - 1.0) * 100.0;

        println!("  SP improvement over MD:  {:.1}%", improvement_sp);
        println!("  WD improvement over MD:  {:.1}%", improvement_wd);

        if wd_score > sp_score {
            let wd_over_sp = (wd_score as f64 / sp_score as f64 - 1.0) * 100.0;
            println!("  ✅ WD is {:.1}% better than SP!", wd_over_sp);
        } else {
            println!("  ⚠️  WD did not improve over SP");
        }
    }
}
