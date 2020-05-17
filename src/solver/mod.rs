use crate::board::{Board, CellLoc};
use std::collections::BTreeSet;
use std::error;
use std::fmt;

mod candidate_cache;
mod generator;
use candidate_cache::CandidateCache;
pub use generator::generate;

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

    fn get_strategy(&self) -> Option<Strategy> {
        match self {
            Self::SetValue { strategy, .. } => Some(*strategy),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
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
pub struct SudokuSolver {
    board: Board,
    // possible_values: HashMap<CellLoc, BTreeSet<u8>>,
    candidate_cache: CandidateCache,
    move_log: Vec<MoveLog>,
}

pub fn solve(board: &Board) -> Result<Board, UnsolvableError> {
    let mut solver = SudokuSolver::new(board);
    solver.solve()?;
    Ok(solver.board)
}

impl SudokuSolver {
    pub fn new(board: &Board) -> Self {
        // let possible_values = SudokuSolver::calculate_possible_values(board);
        let candidate_cache = CandidateCache::from_board(&board);

        let solver = SudokuSolver {
            board: board.clone(),
            move_log: Vec::new(),
            candidate_cache,
        };

        solver
    }

    pub fn solve(&mut self) -> Result<(), UnsolvableError> {
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
        return self
            .candidate_cache
            .possible_values()
            .iter()
            .min_by_key(|(_cell, possibilities)| possibilities.len())
            .map(|(cell, possibilities)| {
                (
                    *cell,
                    *possibilities.iter().next().expect(
                        "Empty possibilities should have been caught while registering a move",
                    ),
                )
            })
            .expect("If the table is full then the method should have finished");
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
            return Ok(());
        } else {
            return self.backtrack().and(Ok(()));
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

            if let Some(Strategy::Guess) = strategy {
                // if possible values is not empty we need to try the remaining guesses
                if !self.candidate_cache.possible_values()[&cell].is_empty() {
                    // remove the current guess from the options as well as removing this cell as a candidate for this value
                    self.candidate_cache.remove_candidate(&value, &cell);

                    // then we try each guess (to_owned is needed here otherwise self would be borrowed for
                    // the entirity of the block)
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
                let possbible_values = cell
                    .get_possible_values(&self.board)
                    .expect("cell was unset therefore the value must be Some");

                self.candidate_cache
                    .reset_candidates(&cell, possbible_values);
            }
        }

        return Err(UnsolvableError);
    }
}

#[cfg(test)]
mod tests {
    use super::{Strategy, SudokuSolver, UnsolvableError};
    use std::collections::HashSet;

    #[test]
    fn naked_singles() {
        let solver = SudokuSolver::new(
            &"
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
            .unwrap(),
        );

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
        let solver = SudokuSolver::new(
            &"
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
            .unwrap(),
        );

        assert_eq!(
            solver.hidden_singles(),
            vec![(solver.board.cell_at(0, 8), 9)].drain(..).collect()
        );
    }

    #[test]
    fn hidden_singles_after_backtrack() {
        let mut solver = SudokuSolver::new(
            &"
        ....
        3...
        ....
        ....
        "
            .parse()
            .unwrap(),
        );

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
        let mut solver = SudokuSolver::new(
            &"
            12..
            3...
            ....
            ....
        "
            .parse()
            .unwrap(),
        );

        assert_eq!(
            solver
                // setting a 4 on this cell removes all possible values at (1, 1)
                .register_move(Strategy::Guess, &solver.board.cell_at(2, 1), 4)
                .unwrap_err(),
            UnsolvableError
        );
    }
}
