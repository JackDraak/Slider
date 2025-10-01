/// Debug Walking Distance extraction logic
///
/// Usage: cargo run --example debug_walking_distance

use slider::model::{EntropyCalculator, PuzzleState, WalkingDistance};

fn main() {
    let puzzle = PuzzleState::new(3).unwrap();
    let wd = WalkingDistance::new(3);

    println!("Solved 3×3 puzzle:");
    println!("WD score: {}", wd.calculate(&puzzle));

    // Make one move and test
    let mut puzzle2 = PuzzleState::new(3).unwrap();
    puzzle2.apply_immediate_move((2, 1));

    println!("\nAfter one move:");
    println!("WD score: {}", wd.calculate(&puzzle2));

    // Print what we expect:
    // A solved 3x3 has tiles arranged:
    // 1 2 3
    // 4 5 6
    // 7 8 _

    // After moving tile 8 right:
    // 1 2 3
    // 4 5 6
    // 7 _ 8

    // Row 2 now has: [tile_7 (target row 2), empty, tile_8 (target row 2)]
    // This should have low walking distance since both tiles belong to row 2
}
