use crate::controller::shuffle_controller::ShuffleController;
use crate::model::{
    AStarSolver, ActualSolutionLength, Difficulty, EntropyCalculator, ManhattanDistance,
    PerformanceMetrics, PerformanceTimer, Position, PuzzleError, PuzzleState,
    ShortestPathHeuristic,
};
use std::time::{Duration, Instant};

/// Combined entropy and performance metrics
#[derive(Debug, Clone, Copy)]
pub struct EntropyMetrics {
    pub manhattan_distance: u32,
    pub shortest_path_heuristic: u32,
    pub actual_solution_length: u32,
    pub performance: PerformanceMetrics,
}

/// Tracks the history of moves for scoring and statistics
#[derive(Debug, Default)]
pub struct MoveHistory {
    move_count: usize,
}

impl MoveHistory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_move(&mut self) {
        self.move_count += 1;
    }

    pub fn move_count(&self) -> usize {
        self.move_count
    }

    pub fn reset(&mut self) {
        self.move_count = 0;
    }
}

/// Auto-solve state
#[derive(Debug)]
pub struct AutoSolveState {
    solution_path: Vec<Position>,
    current_step: usize,
    last_move_time: Instant,
    move_interval: Duration,
}

impl AutoSolveState {
    pub fn new(solution_path: Vec<Position>, move_interval: Duration) -> Self {
        Self {
            solution_path,
            current_step: 0,
            last_move_time: Instant::now(),
            move_interval,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.current_step >= self.solution_path.len()
    }

    pub fn progress(&self) -> (usize, usize) {
        (self.current_step, self.solution_path.len())
    }
}

/// Main game controller that orchestrates gameplay
pub struct GameController {
    state: PuzzleState,
    history: MoveHistory,
    shuffle_controller: ShuffleController,
    entropy_calculator: Box<dyn EntropyCalculator>,
    cached_metrics: Option<EntropyMetrics>,
    state_version: u64, // Increments on every state change
    auto_solve: Option<AutoSolveState>,
}

impl GameController {
    /// Creates a new game controller with the specified grid size
    ///
    /// # Errors
    ///
    /// Returns `PuzzleError` if grid_size is invalid (< 3 or > 22)
    pub fn new(grid_size: usize) -> Result<Self, PuzzleError> {
        Ok(Self {
            state: PuzzleState::new(grid_size)?,
            history: MoveHistory::new(),
            shuffle_controller: ShuffleController::new(grid_size)?,
            entropy_calculator: Box::new(ManhattanDistance),
            cached_metrics: None,
            state_version: 0,
            auto_solve: None,
        })
    }

    /// Invalidates cached metrics when state changes
    fn invalidate_cache(&mut self) {
        self.cached_metrics = None;
        self.state_version += 1;
    }

    /// Returns a reference to the current puzzle state
    pub fn state(&self) -> &PuzzleState {
        &self.state
    }

    /// Returns the current move count
    pub fn move_count(&self) -> usize {
        self.history.move_count()
    }

    /// Sets the entropy calculator to use
    pub fn set_entropy_calculator(&mut self, calculator: Box<dyn EntropyCalculator>) {
        self.entropy_calculator = calculator;
    }

    /// Starts a new game with the specified difficulty
    pub fn new_game(&mut self, difficulty: Difficulty) {
        // Size is guaranteed valid since controller was constructed successfully
        self.state = PuzzleState::new(self.state.size()).expect("valid size");
        self.history.reset();
        self.shuffle_controller.shuffle(
            &mut self.state,
            difficulty,
            self.entropy_calculator.as_ref(),
        );
        self.invalidate_cache();
    }

    /// Handles a player click at the given position
    /// Returns true if a valid move was made
    pub fn handle_click(&mut self, pos: Position) -> bool {
        // Check if there's a tile at the clicked position
        if self.state.tile_at(pos).is_none() {
            return false;
        }

        // Attempt to apply the move (handles both immediate and chain moves)
        if self.state.apply_chain_move(pos) {
            println!("→ Manual move to {:?} (total moves: {})", pos, self.move_count() + 1);
            self.history.record_move();
            self.invalidate_cache();
            true
        } else {
            false
        }
    }

    /// Checks if the puzzle is solved
    pub fn is_solved(&self) -> bool {
        self.state.is_solved()
    }

    /// Returns the current entropy level (using primary calculator)
    pub fn current_entropy(&self) -> u32 {
        self.entropy_calculator.calculate(&self.state)
    }

    /// Returns entropy calculated by all three methods with performance metrics
    /// Results are cached and only recalculated when the puzzle state changes
    pub fn all_entropy_metrics(&mut self) -> EntropyMetrics {
        // Return cached metrics if available
        if let Some(metrics) = self.cached_metrics {
            return metrics;
        }

        // Calculate metrics
        let mut perf = PerformanceMetrics::new();

        // Time Manhattan Distance calculation
        let timer = PerformanceTimer::start();
        let manhattan = ManhattanDistance.calculate(&self.state);
        perf.manhattan_time_micros = timer.elapsed_micros();

        // Time Shortest Path Heuristic calculation
        let timer = PerformanceTimer::start();
        let shortest_path = ShortestPathHeuristic.calculate(&self.state);
        perf.heuristic_time_micros = timer.elapsed_micros();

        // Only calculate actual solution for small puzzles or low entropy
        // to avoid performance issues
        let actual = if self.state.size() <= 4 && manhattan < 50 {
            let timer = PerformanceTimer::start();
            let result = ActualSolutionLength::new().calculate(&self.state);
            perf.actual_time_micros = timer.elapsed_micros();
            result
        } else {
            perf.actual_time_micros = 0; // Not calculated
            999 // Placeholder for "too complex to solve in real-time"
        };

        let metrics = EntropyMetrics {
            manhattan_distance: manhattan,
            shortest_path_heuristic: shortest_path,
            actual_solution_length: actual,
            performance: perf,
        };

        // Cache the result
        self.cached_metrics = Some(metrics);

        metrics
    }

    /// Resets to a new solved puzzle
    pub fn reset(&mut self) {
        // Size is guaranteed valid since controller was constructed successfully
        self.state = PuzzleState::new(self.state.size()).expect("valid size");
        self.history.reset();
        self.invalidate_cache();
        self.auto_solve = None;
    }

    /// Starts auto-solve mode, computing and animating the optimal solution
    /// Returns false if puzzle is already solved or no solution found
    pub fn start_auto_solve(&mut self) -> bool {
        if self.state.is_solved() {
            return false;
        }

        println!("\n=== AUTO-SOLVE START ===");
        println!("Current puzzle state entropy (Manhattan): {}", self.current_entropy());
        println!("Move count: {}", self.move_count());

        let solver = AStarSolver::new();
        if let Some(path) = solver.solve_with_path(&self.state) {
            println!("✓ A* calculated NEW solution path with {} moves", path.len());
            println!("First 5 moves: {:?}", &path[..path.len().min(5)]);

            self.auto_solve = Some(AutoSolveState::new(
                path,
                Duration::from_millis(700), // 0.7 seconds per move
            ));
            true
        } else {
            println!("✗ A* failed to find solution!");
            false
        }
    }

    /// Stops auto-solve mode
    pub fn stop_auto_solve(&mut self) {
        println!("\n=== AUTO-SOLVE STOPPED ===");
        if let Some(progress) = self.auto_solve_progress() {
            println!("Stopped at move {}/{}", progress.0, progress.1);
        }
        self.auto_solve = None;
    }

    /// Returns whether auto-solve is active
    pub fn is_auto_solving(&self) -> bool {
        self.auto_solve.is_some()
    }

    /// Returns auto-solve progress (current_step, total_steps)
    pub fn auto_solve_progress(&self) -> Option<(usize, usize)> {
        self.auto_solve.as_ref().map(|s| s.progress())
    }

    /// Checks if auto-solve has a move ready to execute
    /// Returns the position to move if it's time for the next move
    pub fn get_next_auto_solve_move(&mut self) -> Option<Position> {
        if let Some(ref auto_solve) = self.auto_solve {
            if auto_solve.is_complete() {
                return None;
            }

            let elapsed = auto_solve.last_move_time.elapsed();
            if elapsed >= auto_solve.move_interval {
                return Some(auto_solve.solution_path[auto_solve.current_step]);
            }
        }
        None
    }

    /// Executes an auto-solve move and advances to the next step
    /// Should be called after animation completes
    pub fn apply_auto_solve_move(&mut self, pos: Position) -> bool {
        let should_clear;

        if let Some(ref mut auto_solve) = self.auto_solve {
            if self.state.apply_immediate_move(pos) {
                auto_solve.current_step += 1;
                auto_solve.last_move_time = Instant::now();

                // Check if complete before we drop the borrow
                should_clear = auto_solve.is_complete();
            } else {
                return false;
            }
        } else {
            return false;
        }

        // Now that auto_solve borrow is dropped, we can mutate self again
        self.history.record_move();
        self.invalidate_cache();

        if should_clear {
            self.auto_solve = None;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_controller() {
        let controller = GameController::new(4).unwrap();
        assert_eq!(controller.move_count(), 0);
        assert!(controller.is_solved());
    }

    #[test]
    fn test_new_game_shuffles() {
        let mut controller = GameController::new(4).unwrap();
        controller.new_game(Difficulty::Easy);

        assert_eq!(controller.move_count(), 0); // History reset
        assert!(!controller.is_solved()); // Should be shuffled
    }

    #[test]
    fn test_handle_click_immediate_move() {
        let mut controller = GameController::new(4).unwrap();
        let result = controller.handle_click((3, 2));

        assert!(result);
        assert_eq!(controller.move_count(), 1);
        assert!(!controller.is_solved());
    }

    #[test]
    fn test_handle_click_chain_move() {
        let mut controller = GameController::new(4).unwrap();
        let result = controller.handle_click((3, 0));

        assert!(result);
        assert_eq!(controller.move_count(), 1); // Counts as one move
        assert_eq!(controller.state().empty_position(), (3, 0));
    }

    #[test]
    fn test_handle_click_invalid_move() {
        let mut controller = GameController::new(4).unwrap();
        let result = controller.handle_click((0, 0));

        assert!(!result); // Diagonal, not valid
        assert_eq!(controller.move_count(), 0);
    }

    #[test]
    fn test_handle_click_empty_cell() {
        let mut controller = GameController::new(4).unwrap();
        let empty_pos = controller.state().empty_position();
        let result = controller.handle_click(empty_pos);

        assert!(!result);
        assert_eq!(controller.move_count(), 0);
    }

    #[test]
    fn test_auto_solve_simple_puzzle() {
        let mut controller = GameController::new(3).unwrap();

        // Make a few moves to scramble
        controller.handle_click((2, 1));
        controller.handle_click((2, 0));

        // Start auto-solve
        assert!(controller.start_auto_solve());
        assert!(controller.is_auto_solving());

        // Get expected path length
        let (_, total_steps) = controller.auto_solve_progress().unwrap();

        // Force immediate execution by manipulating time
        // In real usage, update_auto_solve would be called every frame
        // and time would naturally pass
        if let Some(ref mut auto_solve) = controller.auto_solve {
            auto_solve.move_interval = Duration::from_millis(0); // Instant moves for testing
        }

        // Execute all moves
        for _ in 0..=total_steps {
            if let Some(next_move) = controller.get_next_auto_solve_move() {
                controller.apply_auto_solve_move(next_move);
            }
        }

        // Should be solved
        assert!(controller.state().is_solved());
        assert!(!controller.is_auto_solving()); // Completes and stops
    }

    #[test]
    fn test_auto_solve_already_solved() {
        let mut controller = GameController::new(3).unwrap();

        // Try to auto-solve when already solved
        assert!(!controller.start_auto_solve());
        assert!(!controller.is_auto_solving());
    }

    #[test]
    fn test_reset() {
        let mut controller = GameController::new(4).unwrap();
        controller.handle_click((3, 2));
        controller.handle_click((2, 2));

        assert_eq!(controller.move_count(), 2);

        controller.reset();

        assert_eq!(controller.move_count(), 0);
        assert!(controller.is_solved());
    }

    #[test]
    fn test_current_entropy() {
        let mut controller = GameController::new(4).unwrap();
        assert_eq!(controller.current_entropy(), 0);

        controller.handle_click((3, 2));
        assert!(controller.current_entropy() > 0);
    }

    #[test]
    fn test_metrics_caching() {
        let mut controller = GameController::new(4).unwrap();

        // First call calculates and caches
        let metrics1 = controller.all_entropy_metrics();
        let time1 = metrics1.performance.manhattan_time_micros;

        // Second call should return cached (time should be the same)
        let metrics2 = controller.all_entropy_metrics();
        let time2 = metrics2.performance.manhattan_time_micros;

        assert_eq!(time1, time2);
        assert_eq!(metrics1.manhattan_distance, metrics2.manhattan_distance);

        // After a move, cache should invalidate
        controller.handle_click((3, 2));
        let metrics3 = controller.all_entropy_metrics();

        // Values should differ (state changed)
        assert_ne!(metrics1.manhattan_distance, metrics3.manhattan_distance);
    }

    #[test]
    fn test_solve_simple_puzzle() {
        let mut controller = GameController::new(4).unwrap();

        // Make two moves
        controller.handle_click((3, 2));
        controller.handle_click((3, 3));

        // Should be back to solved
        assert!(controller.is_solved());
        assert_eq!(controller.move_count(), 2);
    }
}