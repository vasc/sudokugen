//! Offers a function to help you solve sudoku puzzles.
//! The [`solve`] function takes a sudoku puzzle and returns a new board with
//! the solution, if there is one.
//!
//! ```
//! use sudokugen::board::Board;
//!
//! let mut board: Board =
//!     ". . . | 4 . . | 8 7 .
//!      4 . 3 | . . . | . . .
//!      2 . . | . . 3 | . . 9
//!      ---------------------
//!      . . 6 | 2 . . | . . 7
//!      . . . | 9 . 6 | . . .
//!      3 . 9 | . 8 . | . . .
//!      ---------------------
//!      . . . | . . . | . 4 .
//!      8 7 2 | 5 . . | . . .
//!      . . . | 7 2 . | 6 . .
//!     "
//!        .parse()
//!        .unwrap();
//!
//! board.solve().unwrap();
//! assert_eq!(
//!     board,
//!     "695412873413879526287653419146235987728946135359187264561398742872564391934721658"
//!     .parse()
//!     .unwrap()
//! );
//! ```
//!
//! [`solve`]: fn.solve.html

mod candidate_cache;
pub mod generator;
mod indexed_map;

use crate::board::{Board, CellLoc};
use candidate_cache::CandidateCache;
use indexed_map::Map;
use rand::seq::IteratorRandom;
use std::collections::BTreeSet;
use std::error;
use std::fmt;

#[derive(Debug, Clone, Copy)]
enum Strategy {
    NakedSingle,
    HiddenSingle,
    Guess,
}

#[derive(Debug, Clone)]
enum MoveLog {
    SetValue {
        strategy: Strategy,
        cell: CellLoc,
        value: u8,
        undo_candidates: candidate_cache::UndoSetValue,
    },
}

impl MoveLog {
    fn get_cell(&self) -> CellLoc {
        match self {
            Self::SetValue { cell, .. } => *cell,
        }
    }

    fn get_value(&self) -> u8 {
        match self {
            Self::SetValue { value, .. } => *value,
        }
    }

    fn get_strategy(&self) -> Strategy {
        match self {
            Self::SetValue { strategy, .. } => *strategy,
        }
    }
}

/// An error to represent that this board is not solvable in it's current state
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsolvableError;

impl fmt::Display for UnsolvableError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "The board has no solution")
    }
}

// This is important for other errors to wrap this one.
impl error::Error for UnsolvableError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

#[derive(Debug)]
struct SudokuSolver<'a> {
    board: &'a mut Board,
    candidate_cache: CandidateCache,
    move_log: Vec<MoveLog>,
    random: bool,
}

impl Board {
    /// Solves the sudoku puzzle.
    ///
    /// Updates the current board with the solution to that sudoku puzzle.
    ///
    /// ```
    /// use sudokugen::board::Board;
    ///
    /// let mut board: Board =
    ///     ". . . | 4 . . | 8 7 .
    ///      4 . 3 | . . . | . . .
    ///      2 . . | . . 3 | . . 9
    ///      ---------------------
    ///      . . 6 | 2 . . | . . 7
    ///      . . . | 9 . 6 | . . .
    ///      3 . 9 | . 8 . | . . .
    ///      ---------------------
    ///      . . . | . . . | . 4 .
    ///      8 7 2 | 5 . . | . . .
    ///      . . . | 7 2 . | 6 . .
    ///     "
    ///        .parse()
    ///        .unwrap();
    ///
    /// board.solve().unwrap();
    ///
    /// assert_eq!(
    ///     board,
    ///     "695412873413879526287653419146235987728946135359187264561398742872564391934721658"
    ///     .parse()
    ///     .unwrap()
    /// );
    /// ```
    ///
    /// If the puzzle has no possible solutions, this function returns [`UnsolvableError`].
    ///
    /// ```
    /// # use sudokugen::board::Board;
    /// #
    /// let mut board: Board = "123. ...4 .... ....".parse().unwrap();
    /// assert!(matches!(board.solve(), Err(UnsolvableError)));
    /// ```
    ///
    /// [`board`]: ../board/struct.Board.html
    /// [`UnsolvableError`]: struct.UnsolvableError.html
    pub fn solve(&mut self) -> Result<(), UnsolvableError> {
        let mut solver = SudokuSolver::new(self);
        solver.solve()?;
        Ok(())
    }
}

impl<'a> SudokuSolver<'a> {
    fn new(board: &'a mut Board) -> Self {
        let candidate_cache = CandidateCache::from_board(board);

        SudokuSolver {
            board,
            move_log: Vec::new(),
            candidate_cache,
            random: false,
        }
    }

    fn new_random(board: &'a mut Board) -> Self {
        let mut solver = Self::new(board);
        solver.random = true;
        solver
    }

    fn solve(&mut self) -> Result<(), UnsolvableError> {
        if self
            .candidate_cache
            .possible_values()
            .iter()
            .any(|(_, values)| values.is_empty())
        {
            return Err(UnsolvableError);
        }

        while !self.candidate_cache.possible_values().is_empty() {
            self.solve_iteration()?;
        }
        Ok(())
    }

    fn naked_singles(&self) -> BTreeSet<(CellLoc, u8)> {
        self.candidate_cache
            .possible_values()
            .iter()
            .filter_map(|(cell, values)| match values.len() {
                1 => Some((*cell, *(values.iter().next().unwrap()))),
                _ => None,
            })
            .collect()
    }

    fn hidden_singles(&self) -> BTreeSet<(CellLoc, u8)> {
        self.candidate_cache
            .iter_candidates()
            .filter_map(|candidate| {
                if candidate.cells.len() != 1 {
                    return None;
                }

                Some((*candidate.cells.iter().next().unwrap(), *candidate.value))
            })
            .collect()
    }

    fn guess(&self) -> (CellLoc, u8) {
        let rng = if self.random {
            Some(rand::thread_rng())
        } else {
            None
        };

        self.candidate_cache
            .possible_values()
            .iter()
            .min_by_key(|(_cell, possibilities)| possibilities.len())
            .map(|(cell, possibilities)| {
                let value = rng
                    .and_then(|mut rng| possibilities.iter().choose(&mut rng))
                    .or_else(|| possibilities.iter().next())
                    .expect("Empty possibilities should have been caught while registering a move");

                (*cell, *value)
            })
            .expect("If the table is full then the method should have finished")
    }

    #[cfg(debug)]
    fn assert_possible_values(&self) {
        let gen_possible_values = self.candidate_cache.possible_values();

        if self.possible_values != gen_possible_values {
            for cell in self
                .possible_values
                .keys()
                .chain(gen_possible_values.keys())
            {
                if self.possible_values.get(cell) != gen_possible_values.get(cell) {
                    println!("main {} -> {:?}", cell, self.possible_values.get(cell));
                    println!("cache {} -> {:?}", cell, gen_possible_values.get(cell));
                }
            }
            panic!();
        }
    }

    fn solve_iteration(&mut self) -> Result<(), UnsolvableError> {
        let naked_singles = self.naked_singles();

        if !naked_singles.is_empty() {
            for (cell, value) in naked_singles {
                if let Ok(ref mut moves) = self.register_move(Strategy::NakedSingle, &cell, value) {
                    self.move_log.append(moves);
                } else {
                    return self.backtrack().and(Ok(()));
                }
            }
            return Ok(());
        }

        // Hidden Singles
        let hidden_singles = self.hidden_singles();

        if !hidden_singles.is_empty() {
            for (cell, value) in hidden_singles {
                if let Ok(ref mut moves) = self.register_move(Strategy::HiddenSingle, &cell, value)
                {
                    self.move_log.append(moves);
                } else {
                    return self.backtrack().and(Ok(()));
                }
            }
            return Ok(());
        }

        // Guesses
        let (cell, value) = self.guess();

        if let Ok(ref mut moves) = self.register_move(Strategy::Guess, &cell, value) {
            self.move_log.append(moves);
            Ok(())
        } else {
            self.backtrack().and(Ok(()))
        }
    }

    fn register_move(
        &mut self,
        strategy: Strategy,
        cell: &CellLoc,
        value: u8,
    ) -> Result<Vec<MoveLog>, UnsolvableError> {
        let undo_candidates = self
            .candidate_cache
            .set_value(value, *cell)
            .or(Err(UnsolvableError))?;

        self.board.set(cell, value);

        let log = vec![MoveLog::SetValue {
            strategy,
            cell: *cell,
            value,
            undo_candidates,
        }];

        Ok(log)
    }

    fn undo_move(&mut self, mov: MoveLog) {
        match mov {
            MoveLog::SetValue {
                cell,
                undo_candidates,
                ..
            } => {
                self.board.unset(&cell);
                self.candidate_cache.undo(undo_candidates);
            }
        }
    }

    fn backtrack(&mut self) -> Result<CellLoc, UnsolvableError> {
        while let Some(mov) = self.move_log.pop() {
            let cell = mov.get_cell();
            let value = mov.get_value();
            let strategy = mov.get_strategy();
            self.undo_move(mov);

            if let Strategy::Guess = strategy {
                // if possible values is not empty we need to try the remaining guesses
                if !self
                    .candidate_cache
                    .possible_values()
                    .get(&cell)
                    .unwrap()
                    .is_empty()
                {
                    // remove the current guess from the options as well as removing this cell as a candidate for this value
                    self.candidate_cache.remove_candidate(&value, &cell);

                    // then we try each guess (to_owned is needed here otherwise self would be borrowed for
                    // the entirety of the block)
                    let guesses = self
                        .candidate_cache
                        .possible_values()
                        .get(&cell)
                        .unwrap()
                        .to_owned();
                    for next_guess_value in guesses {
                        // if the move is not immediately rejected
                        if let Ok(ref mut moves) =
                            self.register_move(Strategy::Guess, &cell, next_guess_value)
                        {
                            // guess seems to work for now, lets keep solving
                            self.move_log.append(moves);
                            return Ok(cell);
                        }
                    }
                }

                // none of the possible guesses worked we keep backtracking
                let possible_values = cell
                    .get_possible_values(self.board)
                    .expect("cell was unset therefore the value must be Some");

                self.candidate_cache
                    .reset_candidates(&cell, possible_values);
            }
        }

        Err(UnsolvableError)
    }
}

#[cfg(test)]
mod tests {
    use super::{Strategy, SudokuSolver, UnsolvableError};
    use std::collections::HashSet;

    #[test]
    fn naked_singles() {
        let mut board = "
        12345678.
        2........
        3........
        4........
        5........
        6........
        7.....246
        8.....975
        ......13.
        "
        .parse()
        .unwrap();

        let solver = SudokuSolver::new(&mut board);

        let ns: HashSet<_> = solver.naked_singles().into_iter().collect();
        let res: HashSet<_> = vec![
            (solver.board.cell_at(0, 8), 9),
            (solver.board.cell_at(8, 0), 9),
            (solver.board.cell_at(8, 8), 8),
        ]
        .into_iter()
        .collect();

        assert_eq!(ns, res);
    }

    #[test]
    fn hidden_singles() {
        let mut board = "
        ...45.78.
        9........
        .........
        .........
        .........
        .........
        .........
        .........
        .....9...
        "
        .parse()
        .unwrap();

        let solver = SudokuSolver::new(&mut board);

        assert_eq!(
            solver.hidden_singles(),
            vec![(solver.board.cell_at(0, 8), 9)].drain(..).collect()
        );
    }

    #[test]
    fn hidden_singles_after_backtrack() {
        let mut board = "
        ....
        3...
        ....
        ....
        "
        .parse()
        .unwrap();
        let mut solver = SudokuSolver::new(&mut board);

        let mut log = solver
            .register_move(Strategy::Guess, &solver.board.cell_at(3, 3), 3)
            .unwrap();
        solver.move_log.append(&mut log);

        assert_eq!(
            solver.hidden_singles(),
            vec![
                (solver.board.cell_at(0, 2), 3),
                (solver.board.cell_at(2, 1), 3)
            ]
            .drain(..)
            .collect()
        );

        solver.backtrack().unwrap();

        assert!(solver.hidden_singles().is_empty());
    }

    #[test]
    fn register_move_results_in_error() {
        let mut board = "
        12..
        3...
        ....
        ....
    "
        .parse()
        .unwrap();

        let mut solver = SudokuSolver::new(&mut board);

        assert_eq!(
            solver
                // setting a 4 on this cell removes all possible values at (1, 1)
                .register_move(Strategy::Guess, &solver.board.cell_at(2, 1), 4)
                .unwrap_err(),
            UnsolvableError
        );
    }
}
