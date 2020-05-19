use super::{solve, MoveLog, Strategy, SudokuSolver};
use crate::board::{Board, CellLoc};
use rayon::prelude::*;
use std::collections::{BTreeSet, HashMap};

impl SudokuSolver {}

pub struct GenSudoku {
    board: Board,
    solution: Board,
    guesses: HashMap<CellLoc, BTreeSet<u8>>,
}

pub fn generate(base_size: usize) -> GenSudoku {
    let mut solver = SudokuSolver::new(&Board::new(base_size));
    solver
        .solve()
        .expect("Should always be possible to solve an empty board");

    let non_guesses = solver.move_log.iter().filter_map(|mov| match mov {
        MoveLog::SetValue {
            strategy: Strategy::Guess,
            ..
        } => None,
        MoveLog::SetValue { cell, .. } => Some(cell),
    });

    let mut board = solver.board;

    // remove every cell generated without guessing
    for cell in non_guesses {
        board.unset(cell);
    }

    let minimal_board = remove_false_guesses(board);
    let mut solver = SudokuSolver::new(&minimal_board);
    solver.solve().expect("A generated board must be solvable");
    let givens: BTreeSet<CellLoc> = minimal_board
        .iter_cells()
        .filter(|cell| minimal_board.get(cell).is_some())
        .collect();
    let mut guesses = HashMap::new();
    for mov in solver.move_log {
        // if let Some(Strategy::Guess) = mov.get_strategy() {
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

    GenSudoku {
        board: minimal_board,
        solution: solver.board,
        guesses,
    }
}

impl GenSudoku {
    pub fn board(&self) -> &Board {
        &self.board
    }
    pub fn solution(&self) -> &Board {
        &self.solution
    }

    pub fn is_solution_unique(&self) -> bool {
        for (cell, options) in self.guesses.iter() {
            if options.par_iter().any(|value| {
                let mut board = self.board.clone();
                board.set(cell, *value);
                solve(&board).is_ok()
            }) {
                return false;
            }
        }

        return true;
    }
}

// TODO does not make sense for this to be recursive, removing 1 false guess does not make
// it more likely that other previously kept guesses would turn out to be false
fn remove_false_guesses(board: Board) -> Board {
    let mut cur_board = board.clone();

    for cell in board.iter_cells().filter(|cell| board.get(&cell).is_some()) {
        let mut board = cur_board.clone();

        // this unidiomatic and slightly fragile rust is necessary to avoid cloning
        // board on every loop run
        let value = board.unset(&cell).expect("Guarateed by the loop above");
        let mut possible_values = cell
            .get_possible_values(&board)
            .expect("Guaranteed to be Some by the for loop");
        possible_values.remove(&value);

        if possible_values.par_iter().all(|value| {
            let mut board = board.clone();
            board.set(&cell, *value);
            solve(&board).is_err()
        }) {
            cur_board = board;
        }
    }

    cur_board
}
