use crate::board::{Board, CellLoc};
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::error;
use std::fmt;

mod generator;
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
        options: Option<BTreeSet<u8>>,
    },
    PencilOut(CellLoc, u8),
}

impl MoveLog {
    fn get_cell(&self) -> CellLoc {
        match self {
            Self::SetValue { cell, .. } => *cell,
            Self::PencilOut(cell, _) => *cell,
        }
    }

    fn get_value(&self) -> u8 {
        match self {
            Self::SetValue { value, .. } => *value,
            Self::PencilOut(_, value) => *value,
        }
    }

    fn get_strategy(&self) -> Option<Strategy> {
        match self {
            Self::SetValue { strategy, .. } => Some(*strategy),
            Self::PencilOut(..) => None,
        }
    }
}

#[derive(Debug, Clone)]
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
    possible_values: HashMap<CellLoc, BTreeSet<u8>>,
    move_log: Vec<MoveLog>,
}

pub fn solve(board: &Board) -> Result<Board, UnsolvableError> {
    let mut solver = SudokuSolver::new(board);
    solver.solve()?;
    Ok(solver.board)
}

impl SudokuSolver {
    pub fn new(board: &Board) -> Self {
        let mut solver = SudokuSolver {
            board: board.clone(),
            move_log: Vec::new(),
            possible_values: HashMap::with_capacity(board.get_base_size().pow(4)),
        };

        solver.calculate_possible_values();

        solver
    }

    pub fn solve(&mut self) -> Result<(), UnsolvableError> {
        if self
            .possible_values
            .iter()
            .any(|(_, values)| values.is_empty())
        {
            return Err(UnsolvableError);
        }

        while !self.possible_values.is_empty() {
            self.solve_iteration()?;
        }
        Ok(())
    }

    fn calculate_possible_values(&mut self) {
        for cell in self.board.iter_cells() {
            if let Some(values) = cell.get_possible_values(&self.board) {
                self.possible_values.insert(cell, values);
            } else {
                self.possible_values.remove(&cell);
            }
        }
    }

    fn naked_singles(&self) -> Vec<(CellLoc, u8)> {
        self.possible_values
            .iter()
            .filter_map(|(cell, values)| match values.len() {
                1 => Some((*cell, *(values.iter().next().unwrap()))),
                _ => None,
            })
            .collect()
    }

    fn hidden_singles(&self) -> Vec<(CellLoc, u8)> {
        // TODO: should create a cached structure similar to possible values
        // instead of calculating this one on every loop
        enum HiddenSingle {
            Multiple,
            Single(CellLoc),
        }

        fn insert_hidden_single(
            block: &mut HashMap<usize, HiddenSingle>,
            block_no: usize,
            cell: CellLoc,
        ) {
            if block.get(&block_no).is_none() {
                block.insert(block_no, HiddenSingle::Single(cell));
            } else {
                block.insert(block_no, HiddenSingle::Multiple);
            }
        }

        let mut hidden_singles = Vec::new();

        for value in 1..=self.board.get_base_size().pow(2) as u8 {
            let mut line_block = HashMap::new();
            let mut col_block = HashMap::new();
            let mut square_block = HashMap::new();

            for cell in self.board.iter_cells() {
                let line = cell.line();
                let col = cell.col();
                let square = cell.square();

                if self.board.get(&cell).is_some() {
                    continue;
                }

                if let Some(values) = self.possible_values.get(&cell) {
                    if values.contains(&value) {
                        insert_hidden_single(&mut line_block, line, cell);
                        insert_hidden_single(&mut col_block, col, cell);
                        insert_hidden_single(&mut square_block, square, cell);
                    }
                }
            }

            hidden_singles.append(
                &mut line_block
                    .into_iter()
                    .chain(col_block.into_iter())
                    .chain(square_block.into_iter())
                    .filter_map(|(_, val)| match val {
                        HiddenSingle::Single(cell) if self.board.get(&cell).is_none() => {
                            Some((cell, value))
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>(),
            );
        }

        return hidden_singles;
    }

    fn guess(&self) -> (CellLoc, u8) {
        return self
            .possible_values
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

    fn solve_iteration(&mut self) -> Result<(), UnsolvableError> {
        let naked_singles = self.naked_singles();

        if !naked_singles.is_empty() {
            for (cell, value) in naked_singles {
                if let Ok(ref mut moves) = self.register_move(Strategy::NakedSingle, &cell, value) {
                    self.move_log.append(moves);
                } else {
                    self.backtrack()?;
                    return Ok(());
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
                    self.backtrack()?;
                    return Ok(());
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
            self.backtrack()?;
            return Ok(());
        }
    }

    fn register_move(
        &mut self,
        strategy: Strategy,
        cell: &CellLoc,
        value: u8,
    ) -> Result<Vec<MoveLog>, UnsolvableError> {
        self.board.set(cell, value);

        let options = self.possible_values.remove(&cell);

        let mut log = vec![MoveLog::SetValue {
            strategy,
            cell: *cell,
            value,
            options,
        }];

        for affected_cell in cell
            .iter_line()
            .chain(cell.iter_col())
            .chain(cell.iter_square())
        {
            if let Some(values) = self.possible_values.get_mut(&affected_cell) {
                if values.is_empty() {
                    unreachable!();
                }
                if values.remove(&value) {
                    log.push(MoveLog::PencilOut(affected_cell, value))
                }

                if values.is_empty() {
                    for mov in log {
                        self.undo_move(mov);
                    }

                    return Err(UnsolvableError);
                }
            }
        }

        Ok(log)
    }

    fn undo_move(&mut self, mov: MoveLog) {
        match mov {
            MoveLog::SetValue { cell, options, .. } => {
                self.board.unset(&cell);
                if let Some(options) = options {
                    self.possible_values.insert(cell, options);
                }
            }
            MoveLog::PencilOut(cell, value) => {
                let possibilities = self.possible_values.entry(cell).or_default();
                possibilities.insert(value);
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
                if !self.possible_values[&cell].is_empty() {
                    // first we remove the current guess from the options
                    self.possible_values.entry(cell).and_modify(|options| {
                        options.remove(&value);
                    });

                    // then we try each guess (to_owned is needed here otherwise self would be borrowed for
                    // the entirity of the block)
                    let guesses = self.possible_values.get(&cell).unwrap().to_owned();
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

                // none of the possible guesses worked
                // we keep backtracking
                self.possible_values.insert(
                    cell,
                    cell.get_possible_values(&self.board)
                        .expect("cell was unset therefore the value must be Some"),
                );
            }
        }

        return Err(UnsolvableError);
    }
}

#[cfg(test)]
mod tests {
    use super::SudokuSolver;
    use crate::board::CellLoc;
    use std::collections::HashSet;

    #[test]
    fn hidden_singles() {
        let solver = SudokuSolver::new(
            &"
        123......
        456......
        7........
        .........
        .........
        .........
        .........
        .8.......
        ..9......
        "
            .into(),
        );

        let ns: HashSet<_> = solver.naked_singles().into_iter().collect();
        let res: HashSet<_> = vec![(CellLoc::at(2, 1, 3), 9), (CellLoc::at(2, 2, 3), 8)]
            .into_iter()
            .collect();

        assert_eq!(ns, res);
    }

    #[test]
    fn possible_values_after_parse() {
        let solver = SudokuSolver::new(
            &"...4..87.4.3......2....3..9..62....7...9.6...3.9.8...........4.8725........72.6.."
                .into(),
        );
        for cell in solver.board.iter_cells() {
            if solver.board.get(&cell).is_some() {
                assert_eq!(solver.possible_values.get(&cell), None);
            }
        }
    }

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
            .into(),
        );

        let ns: HashSet<_> = solver.naked_singles().into_iter().collect();
        let res: HashSet<_> = vec![
            (CellLoc::at(0, 8, 3), 9),
            (CellLoc::at(8, 0, 3), 9),
            (CellLoc::at(8, 8, 3), 8),
        ]
        .into_iter()
        .collect();

        assert_eq!(ns, res);
    }
}
