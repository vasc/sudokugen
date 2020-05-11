use colored::Colorize;
use std::collections::BTreeSet;
use std::convert::TryInto;
use std::error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct UnsolvableError;

impl fmt::Display for UnsolvableError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "board has no solution")
    }
}

// This is important for other errors to wrap this one.
impl error::Error for UnsolvableError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

#[derive(Debug, Clone)]
pub struct Board {
    base_size: usize,
    cells: Vec<Option<u8>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CellLoc {
    base_size: usize,
    idx: usize,
}

impl CellLoc {
    pub fn at(l: usize, c: usize, base_size: usize) -> Self {
        CellLoc {
            idx: l * base_size.pow(2) + c,
            base_size,
        }
    }

    pub fn new(idx: usize, base_size: usize) -> Self {
        CellLoc { idx, base_size }
    }

    fn get_index(&self) -> usize {
        self.idx
    }

    // TODO this should probably not be here
    pub fn get_possible_values(&self, board: &Board) -> Option<BTreeSet<u8>> {
        if board.cells[self.idx].is_some() {
            return None;
        }

        Some(self.calculate_possible_values(board))
    }

    fn calculate_possible_values(&self, board: &Board) -> BTreeSet<u8> {
        let mut possible_values: BTreeSet<u8> = (1..=board.base_size.pow(2) as u8).collect();

        let values_iter = self
            .iter_line()
            .chain(self.iter_col())
            .chain(self.iter_square())
            .filter_map(|cell_loc| board.cells[cell_loc.idx]);

        for value in values_iter {
            possible_values.remove(&value);
        }

        possible_values
    }

    pub fn line(&self) -> usize {
        self.idx / self.base_size.pow(2)
    }

    pub fn col(&self) -> usize {
        self.idx % self.base_size.pow(2)
    }

    pub fn square(&self) -> usize {
        let line_no = self.line();
        let col_no = self.col();

        (line_no / self.base_size) * self.base_size + (col_no / self.base_size)
    }

    pub fn iter_line(&self) -> impl Iterator<Item = CellLoc> {
        let base_size = self.base_size;

        let line_start = self.line() * self.base_size.pow(2);
        let line_end = line_start + self.base_size.pow(2);

        (line_start..line_end).map(move |idx| CellLoc { idx, base_size })
    }

    pub fn iter_col(&self) -> impl Iterator<Item = CellLoc> {
        let base_size = self.base_size;
        let col_no = self.col();
        (0..base_size.pow(2)).map(move |line_no| CellLoc {
            idx: line_no * base_size.pow(2) + col_no,
            base_size,
        })
    }

    pub fn iter_square(&self) -> impl Iterator<Item = CellLoc> {
        let base_size = self.base_size;

        let line_no = self.idx / self.base_size.pow(2);
        let col_no = self.idx % self.base_size.pow(2);

        let sq_line = (line_no / base_size) * base_size;
        let sq_col = (col_no / base_size) * base_size;

        (sq_line..(sq_line + base_size)).flat_map(move |line| {
            (sq_col..(sq_col + base_size)).map(move |col| CellLoc {
                idx: line * base_size.pow(2) + col,
                base_size,
            })
        })
    }
}

impl Board {
    #[must_use]
    pub fn new(base_size: usize) -> Self {
        let table = Board {
            base_size,
            cells: vec![None; base_size.pow(4)],
        };

        table
    }

    pub fn get_base_size(&self) -> usize {
        self.base_size
    }

    pub fn set(&mut self, loc: &CellLoc, value: u8) -> Option<u8> {
        self.cells[loc.get_index()].replace(value)
    }

    pub fn set_at(&mut self, l: usize, c: usize, value: u8) -> Option<u8> {
        self.cells[l * 9 + c].replace(value)
    }

    pub fn unset(&mut self, loc: &CellLoc) -> Option<u8> {
        self.cells[loc.get_index()].take()
    }

    #[must_use]
    pub fn get(&self, cell: &CellLoc) -> Option<u8> {
        self.cells[cell.idx]
    }

    #[must_use]
    pub fn get_at(&self, l: usize, c: usize) -> Option<u8> {
        self.cells[l * 9 + c]
    }

    pub fn iter_cells(&self) -> impl Iterator<Item = CellLoc> {
        let base_size = self.base_size;

        (0..self.base_size.pow(4)).map(move |idx| CellLoc { idx, base_size })
    }

    #[must_use]
    pub fn cell_at(&self, idx: usize) -> CellLoc {
        CellLoc {
            idx,
            base_size: self.base_size,
        }
    }

    pub fn print(&self, highlight: Option<CellLoc>) {
        let h_idx = match highlight {
            Some(cell) => cell.idx,
            None => self.base_size.pow(4) + 1,
        };

        for l in 0..9 {
            if l != 0 && l % self.base_size == 0 {
                println!(
                    "{}",
                    (0..self.base_size.pow(2) * 2 + self.base_size - 2)
                        .map(|_| "-")
                        .collect::<String>()
                );
            }
            for c in 0..9 {
                if c != 0 && c % self.base_size == 0 {
                    print!("|")
                }
                if let Some(value) = self.cells[l * 9 + c] {
                    if l * 9 + c == h_idx {
                        print!("{} ", value.to_string().red().bold());
                    } else {
                        print!("{} ", value);
                    }
                } else {
                    print!(". ")
                }
            }
            println!();
        }
    }
}

impl PartialEq for Board {
    fn eq(&self, other: &Self) -> bool {
        if self.base_size != other.base_size {
            return false;
        }

        for idx in 0..self.base_size.pow(4) {
            if self.cells[idx] != other.cells[idx] {
                return false;
            }
        }

        true
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for l in 0..self.base_size.pow(2) {
            for c in 0..self.base_size.pow(2) {
                if let Some(value) = self.cells[l * self.base_size.pow(2) + c] {
                    write!(f, "{} ", value)?;
                } else {
                    write!(f, ". ")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl From<&str> for Board {
    fn from(board_as_string: &str) -> Self {
        let board_as_string = board_as_string.replace(" ", "");
        let board_as_string = board_as_string.replace("\n", "");
        let board_as_string = board_as_string.replace("_", "");
        let board_as_string = board_as_string.replace("|", "");

        let base_size = (board_as_string.len() as f64).sqrt().sqrt();

        if base_size.fract() != 0.0 {
            panic!("String definition of board does not have the correct size")
        }
        let mut table = Board::new(base_size as usize);

        // TODO: must support deserialization of tables larger than base 3
        for (idx, c) in board_as_string.char_indices() {
            match c {
                '1'..='9' => {
                    table.set(
                    &CellLoc::new(idx, base_size as usize),
                    c.to_digit(10)
                        .unwrap()
                        .try_into()
                        .unwrap()
                );
                }
                '.' => continue,
                _ => panic!("All characters in the board representation should be digits or a spacing characted '.', '-', '|' or '\\n'")
            }
        }

        table
    }
}

#[cfg(test)]
mod test {
    use super::Board;
    use super::CellLoc;
    use std::collections::BTreeSet;

    #[test]
    fn basics() {
        let table = Board::new(2);

        assert!(table.iter_cells().all(|cell| table.get(&cell).is_none()));
    }

    #[test]
    fn set_value() {
        let mut table = Board::new(3);
        assert_eq!(table.get_at(0, 0), None);
        table.set(&CellLoc::new(0, 3), 3);
        assert_eq!(table.get_at(0, 0), Some(3));
    }

    #[test]
    fn square() {
        assert_eq!(CellLoc::new(0, 2).square(), 0);
        assert_eq!(CellLoc::new(6, 2).square(), 1);
        assert_eq!(CellLoc::new(14, 2).square(), 3);
    }

    #[test]
    fn iter_cells() {
        let table = Board::new(3);
        assert_eq!(
            table
                .iter_cells()
                .map(|cell| cell.idx)
                .collect::<Vec<usize>>(),
            (0..81).collect::<Vec<usize>>()
        )
    }

    #[test]
    fn iter_square() {
        let cell0 = CellLoc {
            idx: 0,
            base_size: 3,
        };

        assert_eq!(
            cell0
                .iter_square()
                .map(|cell| cell.idx)
                .collect::<Vec<usize>>(),
            &[0, 1, 2, 9, 10, 11, 18, 19, 20]
        )
    }

    #[test]
    fn possible_values_is_zero() {
        let mut table = Board::new(3);
        table.set_at(0, 0, 1);

        let mut iter = table.iter_cells();
        let cell = iter.next().expect("table should have 81 cells");

        assert_eq!(cell.idx, 0);
        assert!(cell.get_possible_values(&table).is_none())
    }

    #[test]
    fn possible_values() {
        let mut table = Board::new(3);
        table.set_at(0, 1, 2);
        table.set_at(0, 2, 3);
        table.set_at(1, 0, 4);
        table.set_at(2, 2, 5);

        let mut iter = table.iter_cells();
        let cell = iter.next().expect("table should have 81 cells");

        assert_eq!(
            cell.get_possible_values(&table),
            Some(
                vec![1u8, 6, 7, 8, 9]
                    .iter()
                    .map(|value| value.to_owned())
                    .collect::<BTreeSet<u8>>()
            )
        )
    }

    #[test]
    fn from() {
        let table = Board::from("................");
        print!("{}", table);
        assert_eq!(table, Board::new(2));
    }
}
