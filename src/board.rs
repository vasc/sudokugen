//! The Board module contains representations of a sudoku [`Board`]
//! as well as the representation of a cell location inside a board
//! [`CellLoc`].
//!
//! Usually you'd use the [`Board`] structure directly to create a board
//! using the [`new`] method or by parsing it from a string. You might also use
//! the [`CellLoc`] structure to reference a location in the board, but
//! the [`cell_at`] method of the board instance is more convenient to address
//! cells of a specific board.
//!
//! [`Board`]: struct.Board.html
//! [`new`]: struct.Board.html#method.new
//! [`CellLoc`]: struct.CellLoc.html
//! [`cell_at`]: struct.Board.html#method.cell_at

use std::collections::BTreeSet;
use std::convert::TryInto;
use std::error;
use std::fmt;
use std::str::FromStr;

/// Represents a sudoku board.
///
/// This is usually the entry point to use any of the functionality in this library.
/// You can create a new board by simply calling new and specifying the base size of the board.
/// ```
/// use sudokugen::board::Board;
/// let board: Board = Board::new(3);
/// ```
///
/// Or you can parse an existing representation of a board using the [`from_str`] method of the [`FromStr`] trait.
///
/// [`FromStr`]: https://doc.rust-lang.org/core/str/trait.FromStr.html
/// [`from_str`]: #method.from_str
///
/// ```
/// use sudokugen::board::Board;
///
/// let board: Board = "
/// 1 . . | . . . | . . .
/// . 2 . | . . . | . . .
/// . . 3 | . . . | . . .
/// ---------------------
/// . . . | 4 . . | . . .
/// . . . | . 5 . | . . .
/// . . . | . . 6 | . . .
/// ---------------------
/// . . . | . . . | 7 . .
/// . . . | . . . | . 8 .
/// . . . | . . . | . . 9
/// ".parse().unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct Board {
    base_size: usize,
    cells: Vec<Option<u8>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// Represents a cell location in the board.
///
/// CellLoc structures are a shallow
/// abstraction of the indice of the cell in the board, using them allows access
/// helper functions to navigate the board and access cell by a more intuitive
/// line/column pair
pub struct CellLoc {
    base_size: usize,
    idx: usize,
}

impl fmt::Display for CellLoc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.line(), self.col())
    }
}

impl fmt::Debug for CellLoc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}]", self.line(), self.col())
    }
}

impl CellLoc {
    /// Returns a cell representing the location at line `l` and column `c`.
    /// The third argument represents the intrinsic size of the board, for a
    /// regular 9 by 9 board the base size is 3 (i.e. sqrt(9))
    ///
    /// ```
    /// use sudokugen::board::CellLoc;
    ///
    /// let cell = CellLoc::at(0, 0, 3);
    /// assert_eq!(cell.line(), 0);
    /// assert_eq!(cell.col(), 0);
    /// ```
    ///
    pub fn at(l: usize, c: usize, base_size: usize) -> Self {
        CellLoc {
            idx: l * base_size.pow(2) + c,
            base_size,
        }
    }

    /// Reference a new location in the board. `idx` is the 0 based flat ordering of all cells
    /// in the board and base_size is the intrinsic size of the board, for a
    /// regular 9 by 9 board the base size is 3 (i.e. sqrt(9)).
    ///
    /// ```
    /// use sudokugen::board::CellLoc;
    ///
    /// let cell = CellLoc::new(9, 3);
    /// assert_eq!((cell.line(), cell.col()), (1, 0));
    /// ```
    pub fn new(idx: usize, base_size: usize) -> Self {
        CellLoc { idx, base_size }
    }

    /// Returns the 0 based flat index of this cell location
    ///
    /// ```
    /// use sudokugen::board::CellLoc;
    ///
    /// let cell = CellLoc::new(9, 3);
    /// assert_eq!(cell.get_index(), 9);
    /// ```
    pub fn get_index(&self) -> usize {
        self.idx
    }

    /// Given a board this returns all the possible values for this cell location
    /// within that board.
    /// If the cell is not empty then it returns `None`
    ///
    /// ```
    /// use sudokugen::board::CellLoc;    
    /// use sudokugen::board::Board;
    ///
    /// let cell = CellLoc::at(0, 1, 2);
    /// let board: Board = "
    /// 1 . | . .
    /// . . | . .
    /// ---------
    /// . 2 | . .
    /// . . | . .
    ///".parse().unwrap();
    /// assert_eq!(cell.get_possible_values(&board), Some(vec![3, 4].into_iter().collect()));
    /// ```
    pub fn get_possible_values(&self, board: &Board) -> Option<BTreeSet<u8>> {
        // TODO this should probably return a result in case of overflow
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

    /// Returns the 0 based line number for this cell location
    ///
    /// ```
    /// use sudokugen::board::CellLoc;
    ///
    /// let cell = CellLoc::at(0, 0, 3);
    /// assert_eq!(cell.line(), 0);
    /// ```
    pub fn line(&self) -> usize {
        self.idx / self.base_size.pow(2)
    }

    /// Returns the 0 based column number for this cell location
    ///
    /// ```
    /// use sudokugen::board::CellLoc;
    ///
    /// let cell = CellLoc::at(0, 0, 3);
    /// assert_eq!(cell.col(), 0);
    /// ```
    pub fn col(&self) -> usize {
        self.idx % self.base_size.pow(2)
    }

    /// Returns the 0 based square number for this cell location.
    /// Squares are numbered line first and then columns.
    ///
    /// ```
    /// use sudokugen::board::CellLoc;
    ///
    /// let cell = CellLoc::at(4, 3, 3);
    /// assert_eq!(cell.square(), 4);
    /// ```
    pub fn square(&self) -> usize {
        let line_no = self.line();
        let col_no = self.col();

        (line_no / self.base_size) * self.base_size + (col_no / self.base_size)
    }

    /// Iterates over all cells in the same line as this one.
    ///
    /// ```
    /// use sudokugen::board::CellLoc;
    ///
    /// let cell = CellLoc::at(0, 0, 2);
    /// assert_eq!(
    ///     cell.iter_line().collect::<Vec<CellLoc>>(),
    ///     vec![
    ///         CellLoc::at(0, 0, 2),
    ///         CellLoc::at(0, 1, 2),
    ///         CellLoc::at(0, 2, 2),
    ///         CellLoc::at(0, 3, 2),
    ///     ]
    ///);
    pub fn iter_line(&self) -> impl Iterator<Item = CellLoc> {
        let base_size = self.base_size;

        let line_start = self.line() * self.base_size.pow(2);
        let line_end = line_start + self.base_size.pow(2);

        (line_start..line_end).map(move |idx| CellLoc { idx, base_size })
    }

    /// Iterates over all cells in the same column as this one.
    ///
    /// ```
    /// use sudokugen::board::CellLoc;
    ///
    /// let cell = CellLoc::at(0, 0, 2);
    /// assert_eq!(
    ///     cell.iter_col().collect::<Vec<CellLoc>>(),
    ///     vec![
    ///         CellLoc::at(0, 0, 2),
    ///         CellLoc::at(1, 0, 2),
    ///         CellLoc::at(2, 0, 2),
    ///         CellLoc::at(3, 0, 2),
    ///     ]
    ///);
    pub fn iter_col(&self) -> impl Iterator<Item = CellLoc> {
        let base_size = self.base_size;
        let col_no = self.col();
        (0..base_size.pow(2)).map(move |line_no| CellLoc {
            idx: line_no * base_size.pow(2) + col_no,
            base_size,
        })
    }

    /// Iterates over all cells in the same square as this one.
    ///
    /// ```
    /// use sudokugen::board::CellLoc;
    ///
    /// let cell = CellLoc::at(0, 0, 2);
    /// assert_eq!(
    ///     cell.iter_square().collect::<Vec<CellLoc>>(),
    ///     vec![
    ///         CellLoc::at(0, 0, 2),
    ///         CellLoc::at(0, 1, 2),
    ///         CellLoc::at(1, 0, 2),
    ///         CellLoc::at(1, 1, 2),
    ///     ]
    ///);
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
    /// Creates a new empty board. The `base_size` parameter represents the size
    /// of the board, base size is the square root of the the width of the board.
    /// so for instance a 9x9 board has a base size of 3, a 16x16 board has a base size
    /// of 4, etc.
    ///
    /// ```
    /// use sudokugen::board::Board;
    /// let board: Board = Board::new(3);
    /// ```
    #[must_use]
    pub fn new(base_size: usize) -> Self {
        Board {
            base_size,
            cells: vec![None; base_size.pow(4)],
        }
    }

    /// Returns the base size of this board, check the documentation of [`new`] for an
    /// explanation of base size.
    ///
    /// [`new`]: #method.new
    ///
    /// ```
    /// use sudokugen::board::Board;
    /// let board: Board = Board::new(3);
    ///
    /// assert_eq!(board.get_base_size(), 3);
    /// ```
    pub fn get_base_size(&self) -> usize {
        self.base_size
    }

    /// Sets the value of a cell in the board using the [`CellLoc`] structure
    /// abstraction. Returns the previous value in this location.
    ///
    /// [`CellLoc`]: struct.CellLoc.html
    ///
    /// ```
    /// use sudokugen::board::Board;
    /// use sudokugen::board::CellLoc;
    ///
    /// let mut board = Board::new(3);
    /// let cell = CellLoc::at(0, 0, 3);
    /// board.set(&cell, 1);
    ///
    /// assert_eq!(board.get(&cell), Some(1));
    /// ```
    pub fn set(&mut self, loc: &CellLoc, value: u8) -> Option<u8> {
        self.cells[loc.get_index()].replace(value)
    }

    /// Convenience method to set a value in the board using line and column indexing.
    /// Returns the previous value in the board.
    ///
    /// ```
    /// use sudokugen::board::Board;
    ///
    /// let mut board = Board::new(3);
    /// board.set_at(0, 0, 1);
    ///
    /// assert_eq!(board.get_at(0, 0), Some(1));
    /// ```
    pub fn set_at(&mut self, l: usize, c: usize, value: u8) -> Option<u8> {
        self.cells[CellLoc::at(l, c, self.base_size).get_index()].replace(value)
    }

    /// Remove a value from the board at this cell and return the previously saved value.
    ///
    /// ```
    /// use sudokugen::board::Board;
    /// use sudokugen::board::CellLoc;
    ///
    /// let mut board: Board = "1... .... .... ....".parse().unwrap();
    /// let cell = CellLoc::at(0, 0, 2);
    ///
    /// let old_value = board.unset(&cell);
    ///
    /// assert_eq!(old_value, Some(1));
    /// assert_eq!(board.get(&cell), None);
    /// ```
    pub fn unset(&mut self, loc: &CellLoc) -> Option<u8> {
        self.cells[loc.get_index()].take()
    }

    /// Returns the value at a cell if there is any or `None` otherwise.
    ///
    /// ```
    /// use sudokugen::board::Board;
    ///
    /// let mut board: Board = "1... .... .... ....".parse().unwrap();
    ///
    /// assert_eq!(board.get(&board.cell_at(0, 0)), Some(1));
    /// assert_eq!(board.get(&board.cell_at(0, 1)), None);
    /// ```
    #[must_use]
    pub fn get(&self, cell: &CellLoc) -> Option<u8> {
        self.cells[cell.idx]
    }

    /// Same as [`get`] but more ergonomic for manual usage. Returns the
    /// value at that position or None if no value is set. See the method
    /// [`CellLoc::at`] for an explanation on the arrangement of lines and columns.
    ///
    /// [`get`]: #method.get
    /// [`CellLoc::at`]: struct.CellLoc.html#method.at
    ///
    /// ```
    /// use sudokugen::board::Board;
    ///
    /// let mut board: Board = "1... .... .... ....".parse().unwrap();
    ///
    /// assert_eq!(board.get_at(0, 0), Some(1));
    /// assert_eq!(board.get_at(0, 1), None);
    /// ```
    pub fn get_at(&self, l: usize, c: usize) -> Option<u8> {
        self.get(&CellLoc::at(l, c, self.base_size))
    }

    /// Return an iterator over all cells in the board.
    ///
    /// ```
    /// use sudokugen::board::Board;
    /// use sudokugen::board::CellLoc;
    /// use std::collections::BTreeSet;
    ///
    /// let board = Board::new(2);
    ///
    /// assert_eq!(
    ///     board.iter_cells().collect::<BTreeSet<CellLoc>>(),
    ///     (0..4).flat_map(|line| (0..4).map(move |col| CellLoc::at(line.clone(), col, 2))).collect::<BTreeSet<CellLoc>>()
    /// );
    /// ```
    ///
    /// Keep in mind that this iterates only over the cell location
    /// not the cell value, in order to access/modify the current value
    /// you'll need to use the [`get`] and [`set`] methods of this board.
    ///
    /// [`get`]: #method.get
    /// [`set`]: #method.set
    pub fn iter_cells(&self) -> impl Iterator<Item = CellLoc> {
        let base_size = self.base_size;

        (0..self.base_size.pow(4)).map(move |idx| CellLoc { idx, base_size })
    }

    /// Convenience method to return a [`CellLoc`] at this position that is compatible
    /// with this board (has the same `base_size`). See more about referencing cells by
    /// line and column using the [`at`] method
    ///
    /// ```
    /// use sudokugen::board::Board;
    ///
    /// let board = Board::new(3);
    /// let cell = board.cell_at(1, 1);
    ///
    /// assert_eq!((cell.line(), cell.col()), (1, 1));
    /// ```
    ///
    /// [`CellLoc`]: struct.CellLoc.html
    /// [`at`]: struct.CellLoc.html#at
    #[must_use]
    pub fn cell_at(&self, l: usize, c: usize) -> CellLoc {
        CellLoc::at(l, c, self.base_size)
    }

    /// Returns a new sudoku [`board`] rotated clockwise by 90deg.
    ///
    /// Valid sudoku puzzles are also valid if rotated 90deg, 180deg and 270deg,
    /// they are the same puzzle, however must people would have trouble realizing that
    /// they are doing the same puzzle. This function provides a cheap way to turn 1 valid
    /// puzzle into 4.
    ///
    /// ```
    /// use sudokugen::board::Board;
    ///
    /// let board: Board = "
    /// 1 2 | . .
    /// 3 4 | . .
    /// ---------
    /// . . | . .
    /// . . | . .
    /// ".parse().unwrap();
    ///
    /// let rotated_board: Board = "
    /// . . | 3 1
    /// . . | 4 2
    /// ---------
    /// . . | . .
    /// . . | . .
    /// ".parse().unwrap();

    ///
    /// assert_eq!(board.rotated(), rotated_board);
    /// ```
    ///
    // [`board`]: #
    pub fn rotated(&self) -> Self {
        let mut board = Board::new(self.base_size);
        let width = self.base_size.pow(2);

        for cell in self.iter_cells() {
            let l = cell.col();
            let c = width - 1 - cell.line();

            if let Some(value) = self.get(&cell) {
                board.set_at(l, c, value);
            }
        }

        board
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

/// Error returned when the representation of the board cannot be parsed successfully.
///
/// Boards have constraints that cannot be represented in easy to transfer formats (such as strings),
/// A 9x9 board for instance must have exactly 81 cells with values ranging between 1 and 9.
/// This error is returned when those constrainst are not met.
#[derive(Debug, Clone)]
pub struct MalformedBoardError;

impl fmt::Display for MalformedBoardError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "This board is not correctly formed")
    }
}

// This is important for other errors to wrap this one.
impl error::Error for MalformedBoardError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

impl FromStr for Board {
    type Err = MalformedBoardError;

    /// Parses a board from a string. A board will be parsed from a string with each digit
    /// representing a value in the board. Separator characters like space ('` `'), newline ('`\n`'),
    /// underscore ('`_`'), dash ('`-`'), and pipe ('`|`') are ignored to allow a more friendly formatting.
    ///
    /// ```
    /// use sudokugen::board::Board;
    /// let board: Board = "
    /// 1 . . | . . . | . . .
    /// . 2 . | . . . | . . .
    /// . . 3 | . . . | . . .
    /// ---------------------
    /// . . . | 4 . . | . . .
    /// . . . | . 5 . | . . .
    /// . . . | . . 6 | . . .
    /// ---------------------
    /// . . . | . . . | 7 . .
    /// . . . | . . . | . 8 .
    /// . . . | . . . | . . 9
    /// ".parse().unwrap();
    /// ```
    ///
    /// Alternatively a more streamelined format can be used, which is the same but without any formatting characters.
    /// ```
    /// use sudokugen::board::Board;
    /// let board: Board = "123456789........................................................................".parse().unwrap();
    /// ```
    ///
    fn from_str(board_as_string: &str) -> Result<Self, Self::Err> {
        let board_as_string = board_as_string.replace(" ", "");
        let board_as_string = board_as_string.replace("\n", "");
        let board_as_string = board_as_string.replace("_", "");
        let board_as_string = board_as_string.replace("-", "");
        let board_as_string = board_as_string.replace("|", "");

        let base_size = (board_as_string.len() as f64).sqrt().sqrt();

        if base_size.fract() != 0.0 {
            return Err(MalformedBoardError);
            // panic!("String definition of board does not have the correct size")
        }
        let mut table = Board::new(base_size as usize);

        // TODO: must support deserialization of tables larger than base 3
        for (idx, c) in board_as_string.char_indices() {
            match c {
                '1'..='9' => {
                    table.set(
                        &CellLoc::new(idx, base_size as usize),
                        c.to_digit(10).unwrap().try_into().unwrap(),
                    );
                }
                '.' => continue,
                _ => return Err(MalformedBoardError), // _ => panic!("All characters in the board representation should be digits or a spacing characted '.', '-', '|' or '\\n'")
            }
        }

        Ok(table)
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
        assert_eq!(CellLoc::at(0, 0, 3).square(), 0);
        assert_eq!(CellLoc::at(0, 3, 3).square(), 1);
        assert_eq!(CellLoc::at(3, 0, 3).square(), 3);
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
        let table: Board = "................".parse().unwrap();
        print!("{}", table);
        assert_eq!(table, Board::new(2));
    }
}
