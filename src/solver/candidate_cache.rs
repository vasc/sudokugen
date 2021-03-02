use super::indexed_map::{Indexed, IndexedMap, Map};
use crate::board::{Board, CellLoc};
use std::fmt;
use std::{
    collections::{BTreeSet, HashMap},
    hash::Hash,
};

#[derive(Hash, Debug, PartialEq, Eq, Copy, Clone)]
pub enum Block {
    Line(usize),
    Col(usize),
    Square(usize),
}

impl Block {
    fn with_value(&self, value: u8) -> (Self, u8) {
        (*self, value)
    }
}

impl CellLoc {
    fn get_blocks_(&self) -> [Block; 3] {
        [
            Block::Line(self.line()),
            Block::Col(self.col()),
            Block::Square(self.square()),
        ]
    }
}

impl Indexed for CellLoc {
    fn idx(&self) -> usize {
        self.get_index()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NoCadidatesLeftError(CellLoc);

impl fmt::Display for NoCadidatesLeftError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "No candidates left for this cell {}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UndoSetValue {
    moves: Vec<(u8, CellLoc, Block)>,
    options: (CellLoc, Option<BTreeSet<u8>>),
    affected_cell_options: Vec<(CellLoc, u8)>,
}

impl UndoSetValue {
    pub fn alternative_options(&self) -> &Option<BTreeSet<u8>> {
        &self.options.1
    }
}

pub struct Candidates<'a> {
    pub value: &'a u8,
    pub block: &'a Block,
    pub cells: &'a BTreeSet<CellLoc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CandidateCache {
    possible_values: IndexedMap<CellLoc, BTreeSet<u8>>,
    candidate_cells: HashMap<(Block, u8), BTreeSet<CellLoc>>,
}

impl CandidateCache {
    pub fn from_board(board: &Board) -> Self {
        let possible_values = Self::calculate_possible_values(board);

        let mut candidate_cache = CandidateCache {
            possible_values,
            candidate_cells: HashMap::with_capacity(board.board_size().get_base_size().pow(4) * 3),
        };

        for cell in candidate_cache.possible_values.keys() {
            let possible_values = candidate_cache.possible_values.get(&cell);

            for value in 1..=(board.board_size().get_base_size() as u8).pow(2) {
                if let Some(possible_values) = possible_values {
                    if possible_values.contains(&value) {
                        for block in &cell.get_blocks_() {
                            candidate_cache
                                .candidate_cells
                                .entry(block.with_value(value))
                                .or_default()
                                .insert(*cell);
                        }
                    }
                }
            }
        }

        candidate_cache
    }

    fn calculate_possible_values(board: &Board) -> IndexedMap<CellLoc, BTreeSet<u8>> {
        let mut possible_values = IndexedMap::new(board.board_size().get_base_size().pow(4));
        for cell in board.iter_cells() {
            if let Some(values) = cell.get_possible_values(&board) {
                possible_values.insert(cell, values);
            }
        }
        possible_values
    }

    pub fn set_value(
        &mut self,
        value: u8,
        cell: CellLoc,
    ) -> Result<UndoSetValue, NoCadidatesLeftError> {
        // remove all possible values for this cell
        let maybe_options = self.possible_values.remove(&cell);
        let mut moves = Vec::new();

        // in this line, column and square this value is no longer relevant so it's removed from cache
        for block in &cell.get_blocks_() {
            let candidates = self.candidate_cells.remove(&block.with_value(value));

            if let Some(candidates) = candidates {
                moves.extend(
                    &mut candidates
                        .iter()
                        .map(|candidate| (value, *candidate, *block)),
                );
            }

            // remove the cell as candidate for all other values in this line, col and square
            if let Some(other_values) = &maybe_options {
                for other_value in other_values {
                    if *other_value != value {
                        if let Some(candidates) = self
                            .candidate_cells
                            .get_mut(&block.with_value(*other_value))
                        {
                            if candidates.remove(&cell) {
                                moves.push((*other_value, cell, *block));
                            }
                        }
                    }
                }
            }
        }

        let mut affected_cell_options = Vec::new();

        let affected_cells = cell
            .iter_line()
            .chain(cell.iter_col())
            .chain(cell.iter_square());

        for affected_cell in affected_cells {
            if let Some(values) = self.possible_values.get_mut(&affected_cell) {
                assert!(!values.is_empty());

                if values.remove(&value) {
                    affected_cell_options.push((affected_cell, value));

                    // for every cell affected by this one (same line, col and square)
                    // that cell is no longer a candidate for this value in all it's blocks
                    for block in &affected_cell.get_blocks_() {
                        if let Some(cells) = self.candidate_cells.get_mut(&block.with_value(value))
                        {
                            if cells.remove(&affected_cell) {
                                moves.push((value, affected_cell, *block));
                            }
                        }
                    }
                }

                if values.is_empty() {
                    self.undo(UndoSetValue {
                        moves,
                        options: (cell, maybe_options),
                        affected_cell_options,
                    });
                    return Err(NoCadidatesLeftError(cell));
                }
            }
        }

        Ok(UndoSetValue {
            moves,
            options: (cell, maybe_options),
            affected_cell_options,
        })
    }

    pub fn reset_candidates(
        &mut self,
        cell: &CellLoc,
        options: BTreeSet<u8>,
    ) -> Option<BTreeSet<u8>> {
        for value in &options {
            self.add_candidate(value, &cell);
        }

        self.possible_values.insert(*cell, options)
    }

    fn add_candidate(&mut self, value: &u8, cell: &CellLoc) {
        for block in &cell.get_blocks_() {
            self.candidate_cells
                .entry(block.with_value(*value))
                .or_default()
                .insert(*cell);
        }
    }

    pub fn remove_candidate(&mut self, value: &u8, cell: &CellLoc) {
        // first remove the value as an option for that cell
        if let Some(options) = self.possible_values.get_mut(cell) {
            if options.remove(value) {
                // if value was an option for that cell then also remove the cell as
                // a candidate for that value in all blocks
                for block in &cell.get_blocks_() {
                    if let Some(cells) = self.candidate_cells.get_mut(&block.with_value(*value)) {
                        cells.remove(cell);
                    }
                }
            }
        }
    }

    pub fn undo(&mut self, undo: UndoSetValue) {
        if let Some(options) = undo.options.1 {
            let cell = undo.options.0;
            self.possible_values.insert(cell, options);
        }

        for (cell, value) in undo.affected_cell_options {
            self.possible_values.entry(cell).or_default().insert(value);
        }

        for (value, cell, block) in undo.moves {
            self.candidate_cells
                .entry(block.with_value(value))
                .or_default()
                .insert(cell);
        }
    }

    pub fn iter_candidates(&self) -> impl Iterator<Item = Candidates> {
        self.candidate_cells
            .iter()
            .map(|((block, value), cells)| Candidates {
                value,
                block,
                cells,
            })
    }

    pub fn possible_values(&self) -> &IndexedMap<CellLoc, BTreeSet<u8>> {
        &self.possible_values
    }

    #[cfg(test)]
    fn candidates_at(&self, block: &Block, value: &u8) -> Option<&BTreeSet<CellLoc>> {
        self.candidate_cells.get(&block.with_value(*value))
    }

    #[cfg(debug)]
    pub fn possible_values_from_candidates(&self) -> HashMap<CellLoc, BTreeSet<u8>> {
        let mut possible_values: HashMap<CellLoc, BTreeSet<u8>> = HashMap::new();

        for (ValueBlock { value, .. }, cells) in &self.candidate_cells {
            for cell in cells {
                possible_values.entry(*cell).or_default().insert(*value);
            }
        }

        possible_values
    }
}

#[cfg(test)]
mod tests {
    use super::Block::{Col, Line, Square};
    use super::CandidateCache;
    use crate::{
        board::{Board, BoardSize, CellLoc},
        solver::indexed_map::Map,
    };
    use std::collections::BTreeSet;

    fn candidate_cache_from_board(board: &Board) -> CandidateCache {
        CandidateCache::from_board(&board)
    }

    fn candidate_cache_from_board_str(board_str: &str) -> CandidateCache {
        candidate_cache_from_board(&(*board_str).parse().unwrap())
    }

    #[test]
    fn test_iter_candidates() {
        let cc = candidate_cache_from_board(&Board::new(BoardSize::NineByNine));

        assert_eq!(cc.iter_candidates().count(), 81 * 3);
        assert_eq!(
            cc.iter_candidates()
                .map(|candidate| *candidate.value)
                .collect::<BTreeSet<u8>>(),
            (1..=9).collect()
        );
    }

    #[test]
    fn possible_values_after_parse() {
        let board =
            "...4..87.4.3......2....3..9..62....7...9.6...3.9.8...........4.8725........72.6.."
                .parse()
                .unwrap();
        let cc = candidate_cache_from_board(&board);
        for cell in board.iter_cells() {
            if board.get(&cell).is_some() {
                assert_eq!(cc.possible_values().get(&cell), None);
            }
        }
    }

    #[test]
    fn possible_locs_after_parse() {
        let cc = candidate_cache_from_board_str(
            "
        1234567..
        456......
        78.......
        2........
        3........
        5........
        6........
        .........
        .........
        ",
        );

        assert_eq!(
            cc.candidates_at(&Line(0), &9),
            Some(
                &vec![
                    CellLoc::at(0, 7, BoardSize::NineByNine),
                    CellLoc::at(0, 8, BoardSize::NineByNine),
                ]
                .drain(..)
                .collect()
            )
        );

        assert_eq!(
            cc.candidates_at(&Col(0), &9),
            Some(
                &vec![
                    CellLoc::at(7, 0, BoardSize::NineByNine),
                    CellLoc::at(8, 0, BoardSize::NineByNine),
                ]
                .into_iter()
                .collect()
            )
        );

        assert_eq!(
            cc.candidates_at(&Square(0), &9),
            Some(
                &vec![CellLoc::at(2, 2, BoardSize::NineByNine)]
                    .drain(..)
                    .collect()
            )
        );
    }

    #[test]
    fn set_value_updates_possible_values() {
        let board = "
        12..
        ....
        ....
        ....
        "
        .parse()
        .unwrap();

        let mut cc = candidate_cache_from_board(&board);

        cc.set_value(3, CellLoc::at(1, 0, BoardSize::FourByFour))
            .unwrap();

        assert_eq!(
            cc.possible_values().get(&board.cell_at(1, 1)),
            Some(&vec![4_u8].into_iter().collect())
        );
    }

    #[test]
    fn set_value_updates_possible_loc() {
        let board: Board = "
        12..
        ....
        ....
        ....
        "
        .parse()
        .unwrap();

        let mut cc = candidate_cache_from_board(&board);

        cc.set_value(3, board.cell_at(3, 0)).unwrap();

        // square already contains 3 therefore it's removed from possible locations
        assert_eq!(cc.candidates_at(&Square(2), &3), None);

        // line already contains 3 therefore it's removed from possible locations
        assert_eq!(cc.candidates_at(&Line(3), &3), None);

        // column already contains 3 therefore it's removed from possible locations
        assert_eq!(cc.candidates_at(&Col(0), &3), None);

        // setting the 3 above removes the other possible location for a 3 in square 0
        assert_eq!(
            cc.candidates_at(&Square(0), &3),
            Some(&vec![board.cell_at(1, 1)].into_iter().collect()),
        );

        // setting a cell removes it as a possibility to all other values in its blocks
        assert!(!cc
            .candidates_at(&Square(2), &4)
            .unwrap()
            .contains(&board.cell_at(3, 0)));
    }

    #[test]
    fn test_set_value_removes_cell_as_candidate_for_other_values() {
        let board: Board = "
        ....
        ....
        ....
        ....
        "
        .parse()
        .unwrap();

        let mut cc = candidate_cache_from_board(&board);

        assert!(cc
            .candidates_at(&Line(0), &1)
            .unwrap()
            .contains(&board.cell_at(0, 0)));

        cc.set_value(2, board.cell_at(0, 0)).unwrap();

        assert!(!cc
            .candidates_at(&Line(0), &1)
            .unwrap()
            .contains(&board.cell_at(0, 0)));
    }

    #[test]
    fn test_undo() {
        let board: Board = "
        ....
        ....
        ....
        ....
        "
        .parse()
        .unwrap();

        let cc = candidate_cache_from_board(&board);
        let mut cc_clone = cc.clone();

        let undo = cc_clone.set_value(2, board.cell_at(3, 2)).unwrap();
        cc_clone.undo(undo);

        assert_eq!(cc, cc_clone);
    }
}
