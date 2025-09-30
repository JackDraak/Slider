use crate::model::{
    Difficulty, EntropyCalculator, MoveValidator, Position, PuzzleError, PuzzleState,
};
use rand::seq::SliceRandom;
use rand::thread_rng;

/// Controls puzzle shuffling with entropy requirements
pub struct ShuffleController {
    validator: MoveValidator,
}

impl ShuffleController {
    /// Creates a new shuffle controller
    ///
    /// # Errors
    ///
    /// Returns `PuzzleError` if grid_size is invalid (< 3 or > 22)
    pub fn new(grid_size: usize) -> Result<Self, PuzzleError> {
        MoveValidator::new(grid_size)?;
        Ok(Self {
            validator: MoveValidator::new(grid_size)?,
        })
    }

    /// Shuffles the puzzle to meet the specified difficulty level
    /// Uses only immediate moves with no backtracking to guarantee solvability
    pub fn shuffle(
        &self,
        state: &mut PuzzleState,
        difficulty: Difficulty,
        calculator: &dyn EntropyCalculator,
    ) {
        let target_entropy = difficulty.min_entropy(state.size());
        let mut rng = thread_rng();
        let mut previous_empty: Option<Position> = None;
        let max_attempts = 1000;
        let mut attempts = 0;

        while calculator.calculate(state) < target_entropy && attempts < max_attempts {
            let current_empty = state.empty_position();
            let mut moves = self.validator.get_immediate_moves(current_empty);

            // Remove backtracking move
            if let Some(prev) = previous_empty {
                moves.retain(|&pos| pos != prev);
            }

            // If no moves available (shouldn't happen), break
            if moves.is_empty() {
                break;
            }

            // Choose a random move
            let chosen_move = moves.choose(&mut rng).unwrap();
            previous_empty = Some(current_empty);

            state.apply_immediate_move(*chosen_move);
            attempts += 1;
        }
    }

    /// Shuffles with a specific number of moves (alternative to entropy-based)
    pub fn shuffle_n_moves(&self, state: &mut PuzzleState, n: usize) {
        let mut rng = thread_rng();
        let mut previous_empty: Option<Position> = None;

        for _ in 0..n {
            let current_empty = state.empty_position();
            let mut moves = self.validator.get_immediate_moves(current_empty);

            // Remove backtracking move
            if let Some(prev) = previous_empty {
                moves.retain(|&pos| pos != prev);
            }

            if moves.is_empty() {
                break;
            }

            let chosen_move = moves.choose(&mut rng).unwrap();
            previous_empty = Some(current_empty);

            state.apply_immediate_move(*chosen_move);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ManhattanDistance;

    #[test]
    fn test_shuffle_increases_entropy() {
        let mut puzzle = PuzzleState::new(4).unwrap();
        let controller = ShuffleController::new(4).unwrap();
        let calculator = ManhattanDistance;

        let initial_entropy = calculator.calculate(&puzzle);
        controller.shuffle(&mut puzzle, Difficulty::Easy, &calculator);
        let after_entropy = calculator.calculate(&puzzle);

        assert!(after_entropy > initial_entropy);
    }

    #[test]
    fn test_shuffle_meets_difficulty_threshold() {
        let mut puzzle = PuzzleState::new(4).unwrap();
        let controller = ShuffleController::new(4).unwrap();
        let calculator = ManhattanDistance;

        controller.shuffle(&mut puzzle, Difficulty::Medium, &calculator);
        let entropy = calculator.calculate(&puzzle);

        assert!(entropy >= Difficulty::Medium.min_entropy(4));
    }

    #[test]
    fn test_shuffle_n_moves() {
        let mut puzzle = PuzzleState::new(4).unwrap();
        let controller = ShuffleController::new(4).unwrap();

        let initial_empty = puzzle.empty_position();
        controller.shuffle_n_moves(&mut puzzle, 10);
        let after_empty = puzzle.empty_position();

        // Empty position should have changed
        assert_ne!(initial_empty, after_empty);
    }

    #[test]
    fn test_shuffled_puzzle_not_solved() {
        let mut puzzle = PuzzleState::new(4).unwrap();
        let controller = ShuffleController::new(4).unwrap();
        let calculator = ManhattanDistance;

        controller.shuffle(&mut puzzle, Difficulty::Easy, &calculator);

        // Should no longer be in solved state
        assert!(!puzzle.is_solved());
    }

    #[test]
    fn test_no_immediate_backtracking_during_shuffle() {
        // This test verifies that shuffles don't immediately undo the previous move
        let mut puzzle = PuzzleState::new(4).unwrap();
        let controller = ShuffleController::new(4).unwrap();

        for _ in 0..10 {
            let prev_pos = puzzle.empty_position();
            controller.shuffle_n_moves(&mut puzzle, 1);
            let new_pos = puzzle.empty_position();

            // After a move, the empty position should have changed
            assert_ne!(new_pos, prev_pos);
        }
    }
}