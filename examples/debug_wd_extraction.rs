/// Debug Walking Distance extraction
use slider::model::{PuzzleState, WalkingDistance};

fn main() {
    let puzzle = PuzzleState::new(3).unwrap();
    let wd = WalkingDistance::new(3);

    println!("Solved 3×3 puzzle:");

    // Manually inspect what a solved puzzle looks like
    // Layout:
    // 1 2 3
    // 4 5 6
    // 7 8 _

    for row in 0..3 {
        print!("Row {}: ", row);
        for col in 0..3 {
            if let Some(tile) = puzzle.tile_at((row, col)) {
                let (target_row, _target_col) = tile.home_position;
                print!("{} ", target_row);
            } else {
                print!("E ");
            }
        }
        println!();
    }

    println!("\nColumn view:");
    for col in 0..3 {
        print!("Col {}: ", col);
        for row in 0..3 {
            if let Some(tile) = puzzle.tile_at((row, col)) {
                let (_target_row, target_col) = tile.home_position;
                print!("{} ", target_col);
            } else {
                print!("E ");
            }
        }
        println!();
    }

    println!("\nFor a SOLVED puzzle:");
    println!("Row 0 should have all tiles targeting row 0: [0,0,0]");
    println!("Row 1 should have all tiles targeting row 1: [1,1,1]");
    println!("Row 2 should have all tiles targeting row 2: [2,2,E]");
    println!("\nEach should have WD = 0");
}
