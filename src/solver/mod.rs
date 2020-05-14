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
        candidates: (
            Option<BTreeSet<CellLoc>>,
            Option<BTreeSet<CellLoc>>,
            Option<BTreeSet<CellLoc>>,
        ),
    },
    PencilOut(CellLoc, u8),
    RemoveCandidate {
        block: Block,
        value: u8,
        cell: CellLoc,
    },
}

impl MoveLog {
    fn get_cell(&self) -> CellLoc {
        match self {
            Self::SetValue { cell, .. } => *cell,
            Self::PencilOut(cell, _) => *cell,
            Self::RemoveCandidate { cell, .. } => *cell,
        }
    }

    fn get_value(&self) -> u8 {
        match self {
            Self::SetValue { value, .. } => *value,
            Self::PencilOut(_, value) => *value,
            Self::RemoveCandidate { value, .. } => *value,
        }
    }

    fn get_strategy(&self) -> Option<Strategy> {
        match self {
            Self::SetValue { strategy, .. } => Some(*strategy),
            _ => None,
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
#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
enum Block {
    Line(usize),
    Col(usize),
    Square(usize),
}

impl CellLoc {
    fn get_blocks(&self) -> [Block; 3] {
        [
            Block::Line(self.line()),
            Block::Col(self.col()),
            Block::Square(self.square()),
        ]
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
struct ValueBlock {
    value: u8,
    block: Block,
}

impl Block {
    fn with_value(&self, value: u8) -> ValueBlock {
        ValueBlock {
            block: *self,
            value,
        }
    }
}

#[derive(Debug)]
pub struct SudokuSolver {
    board: Board,
    possible_values: HashMap<CellLoc, BTreeSet<u8>>,
    candidate_cells: HashMap<ValueBlock, BTreeSet<CellLoc>>,
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
            candidate_cells: HashMap::with_capacity(board.get_base_size().pow(4) * 3),
        };

        solver.calculate_possible_values();
        solver.calculate_candidates();

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

    fn calculate_candidates(&mut self) {
        for cell in self.board.iter_cells() {
            let possible_values = self.possible_values.get(&cell);

            for value in 1..=(self.board.get_base_size() as u8).pow(2) {
                if let Some(possible_values) = possible_values {
                    if possible_values.contains(&value) {
                        for block in &cell.get_blocks() {
                            self.candidate_cells
                                .entry(block.with_value(value))
                                .or_default()
                                .insert(cell);
                        }
                    }
                }
            }
        }
    }

    fn naked_singles(&self) -> BTreeSet<(CellLoc, u8)> {
        self.possible_values
            .iter()
            .filter_map(|(cell, values)| match values.len() {
                1 => Some((*cell, *(values.iter().next().unwrap()))),
                _ => None,
            })
            .collect()
    }

    fn hidden_singles(&self) -> BTreeSet<(CellLoc, u8)> {
        self.candidate_cells
            .iter()
            .filter_map(|(ValueBlock { block: _, value }, cells)| {
                if cells.len() != 1 {
                    return None;
                }

                Some((*cells.iter().next().unwrap(), *value))
            })
            .collect()
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

        // in thie line, column and square this value is no longer relevant so it's removed from cache
        let line_candidates = self
            .candidate_cells
            .remove(&Block::Line(cell.line()).with_value(value));
        let col_candidates = self
            .candidate_cells
            .remove(&Block::Col(cell.col()).with_value(value));
        let square_candidates = self
            .candidate_cells
            .remove(&Block::Square(cell.square()).with_value(value));

        let mut log = vec![MoveLog::SetValue {
            strategy,
            cell: *cell,
            value,
            // unfortunate, but required to avoid overcomplicating the logic
            // so that options can be used later while still adding this log
            // in the beginning of the method
            options: options.clone(),
            candidates: (line_candidates, col_candidates, square_candidates),
        }];

        // possible locations
        // remove this value from possible locations for line, col and square
        for block in &cell.get_blocks() {
            // remove the cell as candidate for all other values in this line, col and square
            if let Some(options) = &options {
                for other_value in options {
                    if *other_value != value {
                        if let Some(candidates) = self
                            .candidate_cells
                            .get_mut(&block.with_value(*other_value))
                        {
                            if candidates.remove(&cell) {
                                log.push(MoveLog::RemoveCandidate {
                                    block: *block,
                                    cell: *cell,
                                    value: *other_value,
                                });
                            }
                        }
                    }
                }
            }
        }

        for affected_cell in cell
            .iter_line()
            .chain(cell.iter_col())
            .chain(cell.iter_square())
        {
            // possible values
            if let Some(values) = self.possible_values.get_mut(&affected_cell) {
                if values.is_empty() {
                    unreachable!();
                }

                if values.remove(&value) {
                    log.push(MoveLog::PencilOut(affected_cell, value));

                    // for the possible locations
                    // if this value is not possible in the cell, then this cell is also
                    // not a candidate for this value in any of it's blocks
                    for block in &affected_cell.get_blocks() {
                        if let Some(cells) = self.candidate_cells.get_mut(&block.with_value(value))
                        {
                            if cells.remove(&affected_cell) {
                                log.push(MoveLog::RemoveCandidate {
                                    block: *block,
                                    cell: affected_cell,
                                    value,
                                });
                            }
                        }
                    }
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
            MoveLog::SetValue {
                cell,
                options,
                candidates,
                value,
                ..
            } => {
                self.board.unset(&cell);

                if let Some(options) = options {
                    self.possible_values.insert(cell, options);
                }

                if let Some(candidates) = candidates.0 {
                    self.candidate_cells
                        .insert(Block::Line(cell.line()).with_value(value), candidates);
                }
                if let Some(candidates) = candidates.1 {
                    self.candidate_cells
                        .insert(Block::Col(cell.col()).with_value(value), candidates);
                }
                if let Some(candidates) = candidates.2 {
                    self.candidate_cells
                        .insert(Block::Square(cell.square()).with_value(value), candidates);
                }
            }
            MoveLog::PencilOut(cell, value) => {
                let possibilities = self.possible_values.entry(cell).or_default();
                possibilities.insert(value);
            }
            MoveLog::RemoveCandidate { block, value, cell } => {
                let possibilities = self
                    .candidate_cells
                    .entry(block.with_value(value))
                    .or_default();
                possibilities.insert(cell);
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

                    // as well as removing this cell as a candidate for this value
                    for block in &cell.get_blocks() {
                        self.candidate_cells
                            .entry(block.with_value(value))
                            .and_modify(|cells| {
                                cells.remove(&cell);
                            });
                    }

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

                // none of the possible guesses worked we keep backtracking
                let possbible_values = cell
                    .get_possible_values(&self.board)
                    .expect("cell was unset therefore the value must be Some");

                // first reset candidates
                for value in &possbible_values {
                    for block in &cell.get_blocks() {
                        self.candidate_cells
                            .entry(block.with_value(*value))
                            .or_default()
                            .insert(cell);
                    }
                }

                // replace possible values cache
                self.possible_values.insert(cell, possbible_values);
            }
        }

        return Err(UnsolvableError);
    }
}

#[cfg(test)]
mod tests {
    use super::{Block, Strategy, SudokuSolver, UnsolvableError};
    use crate::board::CellLoc;
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
    fn possible_values_after_parse() {
        let solver = SudokuSolver::new(
            &"...4..87.4.3......2....3..9..62....7...9.6...3.9.8...........4.8725........72.6.."
                .parse()
                .unwrap(),
        );
        for cell in solver.board.iter_cells() {
            if solver.board.get(&cell).is_some() {
                assert_eq!(solver.possible_values.get(&cell), None);
            }
        }
    }

    #[test]
    fn possible_locs_after_parse() {
        let solver = SudokuSolver::new(
            &"
        1234567..
        456......
        78.......
        2........
        3........
        5........
        6........
        .........
        .........
        "
            .parse()
            .unwrap(),
        );

        assert_eq!(
            solver.candidate_cells.get(&Block::Line(0).with_value(9)),
            Some(
                &vec![CellLoc::at(0, 7, 3), CellLoc::at(0, 8, 3),]
                    .drain(..)
                    .collect()
            )
        );

        assert_eq!(
            solver.candidate_cells.get(&Block::Col(0).with_value(9)),
            Some(
                &vec![CellLoc::at(7, 0, 3), CellLoc::at(8, 0, 3),]
                    .into_iter()
                    .collect()
            )
        );

        assert_eq!(
            solver.candidate_cells.get(&Block::Square(0).with_value(9)),
            Some(&vec![CellLoc::at(2, 2, 3)].drain(..).collect())
        );
    }

    #[test]
    fn register_move_updates_possible_values() {
        let mut solver = SudokuSolver::new(
            &"
        12..
        ....
        ....
        ....
        "
            .parse()
            .unwrap(),
        );

        solver
            .register_move(Strategy::Guess, &CellLoc::at(1, 0, 2), 3)
            .unwrap();

        assert_eq!(
            solver.possible_values.get(&solver.board.cell_at(1, 1)),
            Some(&vec![4_u8].into_iter().collect())
        );
    }

    #[test]
    fn register_move_updates_possible_loc() {
        let mut solver = SudokuSolver::new(
            &"
        12..
        ....
        ....
        ....
        "
            .parse()
            .unwrap(),
        );

        solver
            .register_move(Strategy::Guess, &solver.board.cell_at(3, 0), 3)
            .unwrap();

        // square already contains 3 therefore it's removed from possible locations
        assert!(!solver
            .candidate_cells
            .contains_key(&Block::Square(2).with_value(3)));

        // line already contains 3 therefore it's removed from possible locations
        assert!(!solver
            .candidate_cells
            .contains_key(&Block::Line(3).with_value(3)));

        // column already contains 3 therefore it's removed from possible locations
        assert!(!solver
            .candidate_cells
            .contains_key(&Block::Col(0).with_value(3),));

        // setting the 3 above remove the other possible location for a 3 in square 0
        assert_eq!(
            solver.candidate_cells.get(&Block::Square(0).with_value(3)),
            Some(&vec![solver.board.cell_at(1, 1)].into_iter().collect()),
        );

        // setting a cell removes it as a possibility to all other values in its blocks
        assert!(!solver
            .candidate_cells
            .get(&Block::Square(2).with_value(4))
            .unwrap()
            .contains(&solver.board.cell_at(3, 0)));
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
