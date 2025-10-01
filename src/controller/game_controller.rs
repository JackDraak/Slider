use crate::controller::shuffle_controller::{ShuffleController, ShuffleResult};
use crate::model::{
    AStarSolver, ActualSolutionLength, Difficulty, EntropyCalculator, ManhattanDistance,
    MoveValidator, PerformanceMetrics, PerformanceTimer, Position, PuzzleError, PuzzleState,
    ShortestPathHeuristic,
};
use std::sync::mpsc::{channel, Receiver};
use std::thread::{self, JoinHandle};
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

/// Auto-solve computation state (running in background thread)
pub enum SolverState {
    Computing(JoinHandle<Option<(Vec<Position>, u64)>>, Receiver<()>, bool), // handle, cancel, is_for_autosolve
    Ready(Vec<Position>, u64), // path, solve_time_micros
    Failed,
}

/// Auto-solve state
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
    solver_state: Option<SolverState>,
    last_solve_time_micros: u64, // Performance metric for last A* solve
    last_solution_length: u32, // Actual solution length from last A* solve
    last_shuffle_result: Option<ShuffleResult>, // Track shuffle information
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
            solver_state: None,
            last_solve_time_micros: 0,
            last_solution_length: 0,
            last_shuffle_result: None,
        })
    }

    /// Invalidates cached metrics when state changes
    fn invalidate_cache(&mut self) {
        self.cached_metrics = None;
        self.state_version += 1;
        // Clear solve time and length only if not auto-solving (manual moves invalidate it)
        if !self.is_auto_solving() {
            self.last_solve_time_micros = 0;
            self.last_solution_length = 0;
        }
    }

    /// Returns a reference to the current puzzle state
    pub fn state(&self) -> &PuzzleState {
        &self.state
    }

    /// Returns the current move count
    pub fn move_count(&self) -> usize {
        self.history.move_count()
    }

    /// Returns information about the last shuffle operation
    pub fn last_shuffle_result(&self) -> Option<&ShuffleResult> {
        self.last_shuffle_result.as_ref()
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
        
        // Use shuffle_with_result to track shuffle information
        let shuffle_result = self.shuffle_controller.shuffle_with_result(
            &mut self.state,
            difficulty,
            self.entropy_calculator.as_ref(),
        );
        
        self.last_shuffle_result = Some(shuffle_result);
        self.invalidate_cache();
        self.auto_solve = None;
        self.solver_state = None;

        // Start background computation for actual entropy (metrics only, not auto-solve)
        self.start_background_solve_for_metrics();
    }

    /// Starts background solver for metrics calculation only (not auto-solve)
    fn start_background_solve_for_metrics(&mut self) {
        // Don't compute if already solved or already computing
        if self.state.is_solved() || self.solver_state.is_some() {
            return;
        }

        println!("Computing actual solution length in background...");

        // Clone the state to send to the thread
        let state = self.state.clone();
        let (_cancel_tx, cancel_rx) = channel::<()>();

        // Spawn solver in background thread
        let handle = thread::spawn(move || {
            let timer = PerformanceTimer::start();
            let solver = AStarSolver::new();
            let result = solver.solve_with_path(&state);
            let solve_time = timer.elapsed_micros();

            result.map(|path| (path, solve_time))
        });

        self.solver_state = Some(SolverState::Computing(handle, cancel_rx, false)); // false = not for auto-solve
    }

    /// Handles a player click at the given position
    /// Returns the sequence of immediate moves if valid (for animation)
    /// Returns None if invalid click or no move possible
    /// NOTE: Does NOT apply the moves - presenter must apply them during animation
    pub fn handle_click(&mut self, pos: Position) -> Option<Vec<Position>> {
        // Check if there's a tile at the clicked position
        if self.state.tile_at(pos).is_none() {
            return None;
        }

        // Can't make manual moves during auto-solve
        if self.is_auto_solving() {
            return None;
        }

        // Get the chain move sequence (if valid)
        let validator = MoveValidator::new(self.state.size()).expect("valid size");
        if let Some(moves) = validator.resolve_chain_move(pos, self.state.empty_position()) {
            // Return the sequence WITHOUT applying - presenter will apply during animation
            println!("→ Manual move to {:?} ({} tiles will move)", pos, moves.len());
            Some(moves)
        } else {
            None
        }
    }

    /// Applies a single immediate move (called by presenter after animation)
    pub fn apply_move(&mut self, pos: Position) -> bool {
        self.state.apply_immediate_move(pos)
    }

    /// Completes a move sequence (called after all animations done)
    pub fn complete_move_sequence(&mut self) {
        self.history.record_move();
        self.invalidate_cache();
        println!("  Move complete (total moves: {})", self.move_count());
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

        // Only calculate actual solution for trivial puzzles (very low entropy)
        // to avoid UI hangs. For harder puzzles, use Auto-Solve button.
        let actual = if self.state.size() <= 3 && manhattan <= 5 {
            let timer = PerformanceTimer::start();
            let result = ActualSolutionLength::new().calculate(&self.state);
            perf.actual_time_micros = timer.elapsed_micros();
            result
        } else if self.last_solution_length > 0 {
            // Use cached solution from background thread
            perf.actual_time_micros = self.last_solve_time_micros;
            self.last_solution_length
        } else {
            // Not calculated yet
            perf.actual_time_micros = 0;
            999 // Placeholder for "not calculated"
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
        self.solver_state = None;
    }

    /// Starts auto-solve mode, computing and animating the optimal solution
    /// Spawns a background thread to compute the solution (non-blocking)
    /// Returns true if computation started, false if already solved or computing
    pub fn start_auto_solve(&mut self) -> bool {
        if self.state.is_solved() {
            return false;
        }

        // Check if we have a cached solution ready to use
        if let Some(SolverState::Ready(path, solve_time)) = self.solver_state.take() {
            println!("\n=== AUTO-SOLVE START (using cached solution) ===");
            println!("Cached solution: {} moves", path.len());
            println!("Original solve time: {}", PerformanceMetrics::format_duration(solve_time));

            self.auto_solve = Some(AutoSolveState::new(
                path.clone(),
                Duration::from_millis(700),
            ));

            // Put the cached solution back for future use
            self.solver_state = Some(SolverState::Ready(path, solve_time));
            return true;
        }

        // Clear failed state from previous attempts
        if matches!(self.solver_state, Some(SolverState::Failed)) {
            self.solver_state = None;
        }

        // Already computing or running
        if self.solver_state.is_some() || self.auto_solve.is_some() {
            return false;
        }

        println!("\n=== AUTO-SOLVE START ===");
        println!("Current puzzle state entropy (Manhattan): {}", self.current_entropy());
        println!("Move count: {}", self.move_count());
        println!("Spawning solver thread (may take up to 60 seconds)...");

        // Clone the state to send to the thread
        let state = self.state.clone();

        // Create a channel for cancellation (unused for now, but allows future timeout)
        let (_cancel_tx, cancel_rx) = channel::<()>();

        // Spawn solver in background thread
        let handle = thread::spawn(move || {
            let timer = PerformanceTimer::start();
            let solver = AStarSolver::new();
            let result = solver.solve_with_path(&state);
            let solve_time = timer.elapsed_micros();

            result.map(|path| (path, solve_time))
        });

        self.solver_state = Some(SolverState::Computing(handle, cancel_rx, true)); // true = for auto-solve
        true
    }

    /// Checks if solver thread has completed and transitions state
    /// Should be called each frame to poll for completion
    /// Returns true if solution is ready to start executing
    pub fn update_solver_state(&mut self) -> bool {
        let state = self.solver_state.take();

        match state {
            Some(SolverState::Computing(handle, _cancel_rx, is_for_autosolve)) => {
                // Check if thread is done (non-blocking)
                if handle.is_finished() {
                    match handle.join() {
                        Ok(Some((path, solve_time))) => {
                            println!("✓ A* calculated solution path with {} moves", path.len());
                            if is_for_autosolve {
                                println!("First 5 moves: {:?}", &path[..path.len().min(5)]);
                            }
                            println!("Solve time: {}", PerformanceMetrics::format_duration(solve_time));

                            // Store solve time and solution length for metrics display
                            self.last_solve_time_micros = solve_time;
                            self.last_solution_length = path.len() as u32;

                            // Invalidate cache so GUI shows updated metrics
                            self.cached_metrics = None;

                            // Only transition to auto-solve animation if this was for auto-solve
                            if is_for_autosolve {
                                self.auto_solve = Some(AutoSolveState::new(
                                    path.clone(),
                                    Duration::from_millis(700), // 0.7 seconds per move
                                ));
                                // Cache the solution for reuse
                                self.solver_state = Some(SolverState::Ready(path, solve_time));
                                return true;
                            } else {
                                // Cache for later use
                                self.solver_state = Some(SolverState::Ready(path, solve_time));
                            }
                        }
                        Ok(None) => {
                            println!("✗ A* failed to find solution!");
                            self.solver_state = Some(SolverState::Failed);
                        }
                        Err(_) => {
                            println!("✗ Solver thread panicked!");
                            self.solver_state = Some(SolverState::Failed);
                        }
                    }
                } else {
                    // Still computing, put it back
                    self.solver_state = Some(SolverState::Computing(handle, _cancel_rx, is_for_autosolve));
                }
            }
            Some(other) => {
                // Put back non-computing states
                self.solver_state = Some(other);
            }
            None => {}
        }

        false
    }

    /// Returns true if solver is currently computing in background
    pub fn is_solver_computing(&self) -> bool {
        matches!(self.solver_state, Some(SolverState::Computing(_, _, _)))
    }

    /// Returns true if solver is computing for auto-solve (not just metrics)
    pub fn is_solver_computing_for_autosolve(&self) -> bool {
        matches!(self.solver_state, Some(SolverState::Computing(_, _, true)))
    }

    /// Stops auto-solve mode and cancels any running solver
    pub fn stop_auto_solve(&mut self) {
        println!("\n=== AUTO-SOLVE STOPPED ===");
        if let Some(progress) = self.auto_solve_progress() {
            println!("Stopped at move {}/{}", progress.0, progress.1);
        }
        self.auto_solve = None;
        self.solver_state = None;
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

        assert!(result.is_some());

        // Apply the moves (simulating what presenter does after animation)
        if let Some(moves) = result {
            for move_pos in moves {
                controller.apply_move(move_pos);
            }
            controller.complete_move_sequence();
        }

        assert_eq!(controller.move_count(), 1);
        assert!(!controller.is_solved());
    }

    #[test]
    fn test_handle_click_chain_move() {
        let mut controller = GameController::new(4).unwrap();
        let result = controller.handle_click((3, 0));

        assert!(result.is_some());
        let moves = result.unwrap();
        assert_eq!(moves.len(), 3); // Chain of 3 moves

        // Apply the moves (simulating what presenter does after animation)
        for move_pos in moves {
            controller.apply_move(move_pos);
        }
        controller.complete_move_sequence();

        assert_eq!(controller.move_count(), 1); // Counts as one move
        assert_eq!(controller.state().empty_position(), (3, 0));
    }

    #[test]
    fn test_handle_click_invalid_move() {
        let mut controller = GameController::new(4).unwrap();
        let result = controller.handle_click((0, 0));

        assert!(result.is_none()); // Diagonal, not valid
        assert_eq!(controller.move_count(), 0);
    }

    #[test]
    fn test_handle_click_empty_cell() {
        let mut controller = GameController::new(4).unwrap();
        let empty_pos = controller.state().empty_position();
        let result = controller.handle_click(empty_pos);

        assert!(result.is_none());
        assert_eq!(controller.move_count(), 0);
    }

    #[test]
    fn test_auto_solve_simple_puzzle() {
        let mut controller = GameController::new(3).unwrap();

        // Make a few moves to scramble
        if let Some(moves) = controller.handle_click((2, 1)) {
            for move_pos in moves {
                controller.apply_move(move_pos);
            }
            controller.complete_move_sequence();
        }
        if let Some(moves) = controller.handle_click((2, 0)) {
            for move_pos in moves {
                controller.apply_move(move_pos);
            }
            controller.complete_move_sequence();
        }

        // Start auto-solve (spawns background thread)
        assert!(controller.start_auto_solve());
        assert!(controller.is_solver_computing());

        // Poll until solver completes (should be fast for 3x3)
        let mut attempts = 0;
        while !controller.update_solver_state() && attempts < 1000 {
            std::thread::sleep(Duration::from_millis(10));
            attempts += 1;
        }

        // Should now be auto-solving (solution ready)
        assert!(controller.is_auto_solving());

        // Get expected path length
        let (_, total_steps) = controller.auto_solve_progress().unwrap();

        // Force immediate execution by manipulating time
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

        // Make moves
        if let Some(moves) = controller.handle_click((3, 2)) {
            for move_pos in moves {
                controller.apply_move(move_pos);
            }
            controller.complete_move_sequence();
        }
        if let Some(moves) = controller.handle_click((2, 2)) {
            for move_pos in moves {
                controller.apply_move(move_pos);
            }
            controller.complete_move_sequence();
        }

        assert_eq!(controller.move_count(), 2);

        controller.reset();

        assert_eq!(controller.move_count(), 0);
        assert!(controller.is_solved());
    }

    #[test]
    fn test_current_entropy() {
        let mut controller = GameController::new(4).unwrap();
        assert_eq!(controller.current_entropy(), 0);

        // Make a move
        if let Some(moves) = controller.handle_click((3, 2)) {
            for move_pos in moves {
                controller.apply_move(move_pos);
            }
            controller.complete_move_sequence();
        }

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
        if let Some(moves) = controller.handle_click((3, 2)) {
            for move_pos in moves {
                controller.apply_move(move_pos);
            }
            controller.complete_move_sequence();
        }
        let metrics3 = controller.all_entropy_metrics();

        // Values should differ (state changed)
        assert_ne!(metrics1.manhattan_distance, metrics3.manhattan_distance);
    }

    #[test]
    fn test_solve_simple_puzzle() {
        let mut controller = GameController::new(4).unwrap();

        // Make two moves
        if let Some(moves) = controller.handle_click((3, 2)) {
            for move_pos in moves {
                controller.apply_move(move_pos);
            }
            controller.complete_move_sequence();
        }
        if let Some(moves) = controller.handle_click((3, 3)) {
            for move_pos in moves {
                controller.apply_move(move_pos);
            }
            controller.complete_move_sequence();
        }

        // Should be back to solved
        assert!(controller.is_solved());
        assert_eq!(controller.move_count(), 2);
    }
}
