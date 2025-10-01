use std::fmt;

/// Errors that can occur during puzzle operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PuzzleError {
    /// Grid size is below minimum (3)
    SizeTooSmall { size: usize, min: usize },
    /// Grid size is above maximum (15)
    SizeTooLarge { size: usize, max: usize },
    /// Invalid move attempted
    InvalidMove { position: (usize, usize) },
    /// Tile not found at position
    TileNotFound { position: (usize, usize) },
}

impl fmt::Display for PuzzleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PuzzleError::SizeTooSmall { size, min } => {
                write!(f, "Puzzle size {} is too small (minimum: {})", size, min)
            }
            PuzzleError::SizeTooLarge { size, max } => {
                write!(f, "Puzzle size {} is too large (maximum: {})", size, max)
            }
            PuzzleError::InvalidMove { position } => {
                write!(f, "Invalid move to position ({}, {})", position.0, position.1)
            }
            PuzzleError::TileNotFound { position } => {
                write!(f, "No tile found at position ({}, {})", position.0, position.1)
            }
        }
    }
}

impl std::error::Error for PuzzleError {}

/// Errors that can occur during solving operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SolverError {
    /// Puzzle is unsolvable (timeout or impossible state)
    Unsolvable,
    /// Maximum iterations exceeded
    TimeoutExceeded { max_iterations: usize },
    /// Invalid puzzle state
    InvalidState(String),
}

impl fmt::Display for SolverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SolverError::Unsolvable => {
                write!(f, "Puzzle is unsolvable or too complex")
            }
            SolverError::TimeoutExceeded { max_iterations } => {
                write!(
                    f,
                    "Solver timeout: exceeded {} iterations",
                    max_iterations
                )
            }
            SolverError::InvalidState(msg) => {
                write!(f, "Invalid puzzle state: {}", msg)
            }
        }
    }
}

impl std::error::Error for SolverError {}

/// Errors that can occur during auto-solve operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutoSolveError {
    /// Solver failed
    SolverFailed(SolverError),
    /// Puzzle already solved
    AlreadySolved,
    /// Auto-solve already in progress
    AlreadyInProgress,
}

impl fmt::Display for AutoSolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AutoSolveError::SolverFailed(err) => {
                write!(f, "Auto-solve failed: {}", err)
            }
            AutoSolveError::AlreadySolved => {
                write!(f, "Puzzle is already solved")
            }
            AutoSolveError::AlreadyInProgress => {
                write!(f, "Auto-solve is already in progress")
            }
        }
    }
}

impl std::error::Error for AutoSolveError {}

impl From<SolverError> for AutoSolveError {
    fn from(err: SolverError) -> Self {
        AutoSolveError::SolverFailed(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_puzzle_error_display() {
        let err = PuzzleError::SizeTooSmall { size: 2, min: 3 };
        assert_eq!(
            err.to_string(),
            "Puzzle size 2 is too small (minimum: 3)"
        );

        let err = PuzzleError::SizeTooLarge { size: 16, max: 15 };
        assert_eq!(
            err.to_string(),
            "Puzzle size 16 is too large (maximum: 15)"
        );
    }

    #[test]
    fn test_solver_error_display() {
        let err = SolverError::TimeoutExceeded {
            max_iterations: 100_000,
        };
        assert_eq!(
            err.to_string(),
            "Solver timeout: exceeded 100000 iterations"
        );
    }

    #[test]
    fn test_auto_solve_error_conversion() {
        let solver_err = SolverError::Unsolvable;
        let auto_err: AutoSolveError = solver_err.into();
        assert!(matches!(auto_err, AutoSolveError::SolverFailed(_)));
    }
}
