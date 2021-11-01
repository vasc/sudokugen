//! Provides a function and a utility struct to help you generate new puzzles.
//! The [`generate`] function takes the base size of the board (see [`Board::new`] for
//! an explanation of base size) and returns a unique, minimal puzzle together with
//! the solution for that puzzle.
//!
//! The [`generate`] function returns a [`GenSudoku`] struct from which you can extract the
//! generated puzzle and respective solution, using it's [`board`] and [`solution`] methods
//! respectively.
//!
//! [`generate`]: fn.generate.html
//! [`Board::new`]: ../../board/struct.Board.html#method.new
//! [`GenSudoku`]: struct.GenSudoku.html
//! [`board`]: struct.GenSudoku.html#method.board
//! [`solution`]: struct.GenSudoku.html#method.solution

use super::{MoveLog, Strategy, SudokuSolver};
use crate::board::{Board, BoardSize, CellLoc};
use rayon::prelude::*;
use std::collections::{BTreeSet, HashMap};

/// This structure represents a generated board and its solution
///
/// This struct can only be created by calling the [`generate`] function, which will create
/// a random board with a unique solution.
///
/// [`generate`]: ../fn.generate.html
pub struct Puzzle {
    board: Board,
    solution: Board,
    guesses: HashMap<CellLoc, BTreeSet<u8>>,
}

impl Board {
    /// Generate a new sudoku board with a unique solution.
    ///
    /// This a utility function for generating a new puzzle when you don't care about the details,
    /// it returns a new random board. It's equivalent to calling `Puzzle::generate(base_size).board().clone();`.
    //. See the full generate documentation at [Puzzle::generate]
    ///
    /// ```
    /// use sudokugen::{Board, BoardSize};
    ///
    /// let board = Board::generate(BoardSize::NineByNine);
    ///
    /// println!("{}", board);
    /// ```
    pub fn generate(board_size: BoardSize) -> Self {
        Puzzle::generate(board_size).board
    }
}

impl Puzzle {
    /// Generate a new sudoku puzzle with a unique solution.
    ///
    /// The generate function creates a random board with a unique solution.
    /// It does this by "solving" the empty board using random guesses whenever
    /// it cannot find the correct solution. Once the empty board is solved,
    /// it iterates over each of the guesses and removes it if that guess is the
    /// only valid option for that cell.
    ///
    /// ```
    /// use sudokugen::{Puzzle, BoardSize};
    ///
    /// let puzzle = Puzzle::generate(BoardSize::NineByNine);
    ///
    /// println!("{}", puzzle.board());
    /// println!("{}", puzzle.solution());
    /// ```
    pub fn generate(board_size: BoardSize) -> Puzzle {
        let mut board = Board::new(board_size);
        let mut solver = SudokuSolver::new_random(&mut board);
        solver
            .solve()
            .expect("Should always be possible to solve an empty board");

        // dbg!(&solver.board.to_string());
        let non_guesses = solver.move_log.iter().filter_map(|mov| match mov {
            MoveLog::SetValue {
                strategy: Strategy::Guess,
                ..
            } => None,
            MoveLog::SetValue { cell, .. } => Some(cell),
        });

        // let mut board = solver.board;

        // remove every cell generated without guessing
        for cell in non_guesses {
            board.unset(cell);
        }

        // let minimal_board = remove_false_guesses(board);
        remove_false_guesses(&mut board);
        let minimal_board = board;

        let mut solved_board = minimal_board.clone();
        let mut solver = SudokuSolver::new(&mut solved_board);
        solver.solve().expect("A generated board must be solvable");
        let givens: BTreeSet<CellLoc> = minimal_board
            .iter_cells()
            .filter(|cell| minimal_board.get(cell).is_some())
            .collect();
        let mut guesses = HashMap::new();
        for mov in solver.move_log {
            if let MoveLog::SetValue {
                cell,
                value,
                strategy: Strategy::Guess,
                undo_candidates,
                ..
            } = mov
            {
                if !givens.contains(&cell) {
                    let mut options = undo_candidates
                        .alternative_options()
                        .as_ref()
                        .unwrap()
                        .to_owned();
                    options.remove(&value);

                    guesses.insert(cell, options);
                }
            }
        }

        Self {
            board: minimal_board,
            solution: solved_board,
            guesses,
        }
    }
    /// Returns the minimal board generated
    ///
    /// ```
    /// use sudokugen::{Puzzle, BoardSize};
    ///
    /// let gen = Puzzle::generate(BoardSize::NineByNine);
    /// println!("{}", gen.board());
    /// ```
    pub fn board(&self) -> &Board {
        &self.board
    }

    /// Returns the solution for the generated board
    ///
    /// ```
    /// use sudokugen::{Puzzle, BoardSize};
    ///
    /// let gen = Puzzle::generate(BoardSize::NineByNine);
    /// println!("{}", gen.solution());
    /// ```
    pub fn solution(&self) -> &Board {
        &self.solution
    }

    /// Verify that the solution for the generated board is unique.
    ///
    /// ```
    /// use sudokugen::{Puzzle, BoardSize};
    ///
    /// let gen = Puzzle::generate(BoardSize::NineByNine);
    /// assert!(gen.is_solution_unique());
    /// ```
    pub fn is_solution_unique(&self) -> bool {
        for (cell, options) in self.guesses.iter() {
            let has_other_solutions = options.par_iter().any(|value| {
                let mut board = self.board.clone();
                board.set(cell, *value);
                board.solve().is_ok()
            });

            if has_other_solutions {
                return false;
            }
        }

        true
    }
}

fn remove_false_guesses(board: &mut Board) {
    // let mut cur_board = board.clone();

    let cells: Vec<_> = board
        .iter_cells()
        .filter(|cell| board.get(cell).is_some())
        .collect();

    for cell in cells {
        // let mut board = cur_board.clone();

        // this unidiomatic and slightly fragile rust is necessary to avoid cloning
        // the board on every loop run
        let value = board.unset(&cell).expect("Guaranteed by the loop above");
        let mut possible_values = cell
            .get_possible_values(board)
            .expect("Guaranteed to be Some by the for loop");
        possible_values.remove(&value);

        let is_guess = possible_values.par_iter().any(|other_value| {
            let mut new_board = board.clone();
            new_board.set(&cell, *other_value);
            new_board.solve().is_ok()
        });

        if is_guess {
            // board was solvable with a different value, this is a legitimate guess, reset it
            board.set(&cell, value);
        }
    }
}
