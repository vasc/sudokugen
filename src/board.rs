// use itertools::Itertools;
use rand::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::convert::TryInto;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Board {
    base_size: usize,
    cells: Vec<Cell>,
}

#[derive(Debug, Clone, Copy)]
struct Cell {
    value: Option<u8>,
    line: usize,
    column: usize,
    square: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CellLoc {
    base_size: usize,
    idx: usize,
}

impl CellLoc {
    pub fn new(idx: usize, base_size: usize) -> Self {
        CellLoc { idx, base_size }
    }
    fn get_index(&self) -> usize {
        self.idx
    }
    // fn try_fill(&self, board: &mut Board) -> Option<u8> {
    //     if board.cells[self.get_index()].value.is_some() {
    //         // value is already set
    //         return None;
    //     }

    //     if let Some(possible_values) = self.possible_values(board) {
    //         let value = possible_values.iter().next().copied();
    //         board.cells[self.get_index()].value = value;
    //         value
    //     } else {
    //         None
    //     }
    // }

    fn possible_values(&self, board: &Board) -> Option<HashSet<u8>> {
        if board.cells[self.idx].value.is_some() {
            return None;
        }

        let mut possible_values: HashSet<u8> =
            (1..=board.base_size.pow(2) as u8).into_iter().collect();

        let values_iter = self
            .iter_line()
            .chain(self.iter_col())
            .chain(self.iter_square())
            .filter_map(|cell_loc| board.cells[cell_loc.idx].value);
        for value in values_iter {
            possible_values.remove(&value);
        }

        Some(possible_values)
    }

    fn line(&self) -> usize {
        self.idx / self.base_size.pow(2)
    }

    fn col(&self) -> usize {
        self.idx % self.base_size.pow(2)
    }

    fn square(&self) -> usize {
        let line_no = self.line();
        let col_no = self.col();

        (line_no / self.base_size) * self.base_size + (col_no / self.base_size)
    }

    fn iter_line(&self) -> impl Iterator<Item = CellLoc> {
        let base_size = self.base_size;

        let line_start = self.line() * self.base_size.pow(2);
        let line_end = line_start + self.base_size.pow(2);

        (line_start..line_end).map(move |idx| CellLoc { idx, base_size })
    }

    fn iter_col(&self) -> impl Iterator<Item = CellLoc> {
        let base_size = self.base_size;
        let col_no = self.col();
        (0..base_size.pow(2)).map(move |line_no| CellLoc {
            idx: line_no * base_size.pow(2) + col_no,
            base_size,
        })
    }

    fn iter_square(&self) -> impl Iterator<Item = CellLoc> {
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
    pub fn new(base_size: usize) -> Self {
        let mut table = Board {
            base_size,
            cells: Vec::with_capacity(base_size.pow(4)),
        };
        for i in 0..base_size.pow(4) {
            let l = i / base_size.pow(2);
            let c = i % base_size.pow(2);
            let s = (i % base_size.pow(2)) / base_size + ((i / base_size.pow(3)) * base_size);
            let cell = Cell {
                value: None,
                line: l,
                column: c,
                square: s,
            };
            table.cells.push(cell);
        }
        table
    }

    fn fitness_line(&self, l: usize) -> u32 {
        let mut line = HashSet::new();
        let mut fitness = 0;
        for idx in l * 9..(l * 9 + 9) {
            if let Some(value) = self.cells[idx].value {
                if line.contains(&value) {
                    fitness += 1;
                };
                line.insert(value);
            }
        }
        fitness
        // check that there are no overlaps in line
    }

    fn fitness_column(&self, c: usize) -> u32 {
        let mut column = HashSet::new();
        let mut fitness = 0;
        for l in 0..9 {
            let cell = &self.cells[l * 9 + c];
            if let Some(value) = cell.value {
                if column.contains(&value) {
                    fitness += 1;
                };
                column.insert(value);
            }
        }
        fitness
    }

    fn fitness_square(&self, s: usize) -> u32 {
        let l0 = (s / 3) * 3;
        let c0 = (s % 3) * 3;
        let mut square = HashSet::new();
        let mut fitness = 0;
        for l in l0..(l0 + 3) {
            for c in c0..(c0 + 3) {
                let cell = &self.cells[l * 9 + c];
                if let Some(value) = cell.value {
                    if square.contains(&value) {
                        fitness += 1;
                    };
                    square.insert(value);
                }
            }
        }
        fitness
    }

    pub fn genes(&self) -> Vec<u32> {
        let mut genes = Vec::with_capacity(27);

        for l in 0..9 {
            genes.push(self.fitness_line(l));
        }

        for c in 0..9 {
            genes.push(self.fitness_column(c));
        }

        for s in 0..9 {
            genes.push(self.fitness_square(s));
        }

        genes
    }

    pub fn fitness(&self) -> u32 {
        let mut fitness = 0;

        for l in 0..9 {
            fitness += self.fitness_line(l);
        }

        for c in 0..9 {
            fitness += self.fitness_column(c);
        }

        for s in 0..9 {
            fitness += self.fitness_square(s);
        }

        return fitness;
    }

    pub fn set(&mut self, loc: CellLoc, value: u8) {
        self.cells[loc.get_index()].value = Some(value);
    }

    pub fn unset(&mut self, loc: CellLoc) {
        self.cells[loc.get_index()].value = None;
    }

    pub fn set_line(&self, line_no: usize, idx: usize, value: u8) -> Board {
        let mut new_table = self.clone();
        new_table.set_line_mut(line_no, idx, value);
        new_table
    }

    pub fn set_column(&self, column_no: usize, idx: usize, value: u8) -> Board {
        let mut new_table = self.clone();
        new_table.set_column_mut(column_no, idx, value);
        new_table
    }

    pub fn set_square(&self, square_no: usize, idx: usize, value: u8) -> Board {
        let mut new_table = self.clone();
        new_table.set_square_mut(square_no, idx, value);
        new_table
    }

    pub fn set_line_mut(&mut self, line_no: usize, idx: usize, value: u8) -> () {
        self.cells[line_no * 9 + idx].value = Some(value);
    }

    pub fn set_column_mut(&mut self, col_no: usize, idx: usize, value: u8) -> () {
        self.cells[idx * 9 + col_no].value = Some(value);
    }

    pub fn set_square_mut(&mut self, square_no: usize, idx: usize, value: u8) -> () {
        let line_0 = (square_no / 3) * 3;
        let col_0 = (square_no % 3) * 3;

        let line = line_0 + idx / 3;
        let col = col_0 + idx % 3;

        self.set_line_mut(line, col, value);
    }

    pub fn set_random(&mut self) {
        let mut rng = thread_rng();
        let mut cell = self.cells.choose_mut(&mut rng).unwrap();
        cell.value = Some(rng.gen_range(1, 10));
    }

    pub fn get(&self, cell: &CellLoc) -> Option<u8> {
        self.cells[cell.idx].value
    }

    pub fn get_at(&self, l: usize, c: usize) -> Option<u8> {
        self.cells[l * 9 + c].value
    }

    pub fn iter_cells(&self) -> impl Iterator<Item = CellLoc> {
        let base_size = self.base_size;

        (0..self.base_size.pow(4)).map(move |idx| CellLoc { idx, base_size })
    }

    pub fn cell_at(&self, idx: usize) -> CellLoc {
        CellLoc {
            idx,
            base_size: self.base_size,
        }
    }

    // fn iter_line(&self, line_no: usize) -> impl Iterator<Item = CellLoc> {
    //     let base_size = self.base_size;
    //     let start = line_no * self.base_size.pow(2);
    //     let end = start + self.base_size.pow(2);
    //     (start..end).map(move |idx| CellLoc { idx, base_size })
    // }

    // fn iter_col(&self, col_no: usize) -> impl Iterator<Item = CellLoc> {
    //     let base_size = self.base_size;

    //     (0..base_size.pow(2)).map(move |line_no| CellLoc {
    //         idx: line_no * base_size + col_no,
    //         base_size,
    //     })
    // }

    pub fn solve(&mut self) -> bool {
        // is there a cell with only 1 possibility?

        // Naked Singles
        let mut possible_values_cache = HashMap::new();
        for cell in self.iter_cells() {
            if let Some(values) = cell.possible_values(self) {
                if values.len() == 0 {
                    return false;
                } else if values.len() == 1 {
                    let value = values.iter().next().unwrap().to_owned();
                    // println!("[Naked Single] Setting cell {:?} to value {}", cell, value);
                    self.set(cell, value);

                    return if self.solve() {
                        true
                    } else {
                        self.unset(cell);
                        false
                    };
                }

                possible_values_cache.insert(cell, values);
            }
        }

        // Hidden Singles
        enum HiddenSingle {
            Multiple,
            Single(CellLoc),
        }

        fn insert_hidden_single(
            block: &mut HashMap<usize, HiddenSingle>,
            block_no: usize,
            cell: CellLoc,
        ) -> () {
            if block.get(&block_no).is_none() {
                block.insert(block_no, HiddenSingle::Single(cell));
            } else {
                block.insert(block_no, HiddenSingle::Multiple);
            }
        }

        for value in 1..=self.base_size.pow(2) as u8 {
            let mut line_block = HashMap::new();
            let mut col_block = HashMap::new();
            let mut square_block = HashMap::new();

            for cell in self.iter_cells() {
                let line = cell.line();
                let col = cell.col();
                let square = cell.square();

                if self.get(&cell).is_some() {
                    continue;
                }

                if let Some(values) = possible_values_cache.get(&cell) {
                    if values.contains(&value) {
                        insert_hidden_single(&mut line_block, line, cell);
                        insert_hidden_single(&mut col_block, col, cell);
                        insert_hidden_single(&mut square_block, square, cell);
                    }
                }
            }

            let mut updated = Vec::new();
            for (_, val) in line_block
                .iter()
                .chain(col_block.iter())
                .chain(square_block.iter())
            {
                if let HiddenSingle::Single(cell) = val {
                    // println!(
                    //     "[Hidden Single] Setting cell ({}, {}) to value {}",
                    //     cell.line(),
                    //     cell.col(),
                    //     value
                    // );
                    self.set(cell.to_owned(), value);
                    updated.push(cell);
                }
            }

            if updated.len() > 0 {
                return if self.solve() {
                    true
                } else {
                    for cell in updated {
                        self.unset(cell.to_owned());
                    }
                    false
                };
            }
        }

        // Guesses
        let mut possible_values = possible_values_cache.into_iter().collect::<Vec<_>>();
        possible_values.sort_by_key(|(_, values)| values.len());

        if let Some((cell, possibilities)) = possible_values.get(0) {
            for value in possibilities {
                self.set(cell.to_owned(), value.to_owned());
                // println!(
                //     "[Guess] Setting cell ({}, {}) to value {}",
                //     cell.line(),
                //     cell.col(),
                //     value
                // );
                if self.solve() {
                    return true;
                }
                // println!(
                //     "[Backtrack] Setting cell ({}, {}) to value None",
                //     cell.line(),
                //     cell.col()
                // );
                self.unset(cell.to_owned());
            }
        }

        self.cells.iter().all(|cell| cell.value.is_some())
    }

    pub fn print(&self) {
        for l in 0..9 {
            for c in 0..9 {
                // let cell = &self.cells[l * 9 + c];
                // print!("{} ", cell.square);

                if let Some(value) = self.cells[l * 9 + c].value {
                    print!("{} ", value);
                } else {
                    print!("_ ")
                }
            }
            print!("\n");
        }
    }
}

impl PartialEq for Board {
    fn eq(&self, other: &Self) -> bool {
        if self.base_size != other.base_size {
            return false;
        }

        for idx in 0..self.base_size.pow(4) {
            if self.cells[idx].value != other.cells[idx].value {
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
                if let Some(value) = self.cells[l * self.base_size.pow(2) + c].value {
                    write!(f, "{} ", value)?;
                } else {
                    write!(f, ". ")?;
                }
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

// pub struct IterLine<'a> {
//     line_no: usize,
//     next: Option<usize>,
//     table: &'a Board,
// }

// impl Board {
//     pub fn iter_line(&self, line_no: usize) -> IterLine {
//         IterLine {
//             line_no: line_no,
//             next: Some(0),
//             table: self,
//         }
//     }
// }

// impl Iterator for IterLine<'_> {
//     type Item = Option<u8>;
//     fn next(&mut self) -> Option<Self::Item> {
//         self.next.map(|col| {
//             self.next = if col < 8 { Some(col + 1) } else { None };
//             self.table.get(self.line_no, col)
//         })
//     }
// }

impl From<&str> for Board {
    fn from(board_as_string: &str) -> Self {
        let base_size = (board_as_string.len() as f64).sqrt().sqrt();

        if base_size.fract() != 0.0 {
            panic!("String definition of board does not have the correct size")
        }

        let mut table = Board::new(base_size as usize);

        for (idx, c) in board_as_string.char_indices() {
            if c != '.' {
                table.set(
                    CellLoc {
                        idx,
                        base_size: base_size as usize,
                    },
                    c.to_digit(10)
                        .expect(
                            "All characters in the board representation should be digits or a '.'",
                        )
                        .try_into()
                        .expect("All numbers in the board should be smaller than 256"),
                )
            }
        }

        table
    }
}

#[cfg(test)]
mod test {
    use super::Board;
    use super::CellLoc;
    use std::collections::HashSet;

    #[test]
    fn basics() {
        let table = Board::new(3);

        assert_eq!(table.fitness(), 0);
    }

    #[test]
    fn set_value() {
        let table = Board::new(3);
        assert_eq!(table.get_at(0, 0), None);
        let table = table.set_line(0, 0, 3);
        assert_eq!(table.get_at(0, 0), Some(3));
    }

    #[test]
    fn set_square() {
        let mut table = Board::new(3);
        // mapping of line numbers and column numbers per square
        //   0 1 2 3 4 5 6 7 8
        // 0 0 0 0 1 1 1 2 2 2
        // 1 0 0 0 1 1 1 2 2 2
        // 2 0 0 0 1 1 1 2 2 2
        // 3 3 3 3 4 4 4 5 5 5
        // 4 3 3 3 4 4 4 5 5 5
        // 5 3 3 3 4 4 4 5 5 5
        // 6 6 6 6 7 7 7 8 8 8
        // 7 6 6 6 7 7 7 8 8 8
        // 8 6 6 6 7 7 7 8 8 8

        table.set_square_mut(1, 4, 0);
        table.set_square_mut(2, 3, 0);
        table.set_square_mut(4, 8, 0);
        table.set_square_mut(8, 5, 0);

        assert_eq!(table.get_at(1, 4), Some(0));
        assert_eq!(table.get_at(1, 6), Some(0));
        assert_eq!(table.get_at(5, 5), Some(0));
        assert_eq!(table.get_at(7, 8), Some(0));
    }

    #[test]
    fn fitness() {
        let table = Board::new(3);
        assert_eq!(table.fitness(), 0);
        let table = table.set_line(0, 0, 1);
        assert_eq!(table.fitness(), 0);
        let table = table.set_line(0, 1, 1);
        let table = table.set_line(1, 0, 1);

        assert_eq!(table.fitness(), 4);
    }

    // #[test]
    // fn iter_line() {
    //     let table = Board::new(3);
    //     let table = table.set_line(0, 0, 3);

    //     let mut iterator = table.iter_line(0);
    //     assert_eq!(iterator.next(), Some(Some(3)));
    //     for val in iterator {
    //         assert_eq!(val, None)
    //     }
    // }

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
        let table = Board::new(3);
        let table = table.set_line(0, 0, 1);

        let mut iter = table.iter_cells();
        let cell = iter.next().expect("table should have 81 cells");

        assert_eq!(cell.idx, 0);
        assert!(cell.possible_values(&table).is_none())
    }

    #[test]
    fn possible_values() {
        let table = Board::new(3);
        let table = table.set_line(0, 1, 2);
        let table = table.set_line(0, 2, 3);
        let table = table.set_line(1, 0, 4);
        let table = table.set_line(2, 2, 5);

        let mut iter = table.iter_cells();
        let cell = iter.next().expect("table should have 81 cells");

        assert_eq!(
            cell.possible_values(&table),
            Some(
                vec![1u8, 6, 7, 8, 9]
                    .iter()
                    .map(|value| value.to_owned())
                    .collect::<HashSet<u8>>()
            )
        )
    }

    #[test]
    fn solve() {
        let mut table = Board::from(
            "...4..87.4.3......2....3..9..62....7...9.6...3.9.8...........4.8725........72.6..",
        );
        table.solve();

        print!("{}", table);

        assert_eq!(
            table,
            Board::from(
                "695412873413879526287653419146235987728946135359187264561398742872564391934721658"
            )
        );
    }

    #[test]
    fn solve2() {
        let mut table = Board::from(
            ".724..3........49.........2921...5.7..4.6...3......2...4..7.....3..196....5..4.21",
        );
        table.solve();

        print!("{}", table);

        assert_eq!(
            table,
            Board::from(
                "572491386318726495469583172921348567754962813683157249146275938237819654895634721"
            )
        );
    }

    #[test]
    fn from() {
        let table = Board::from("................");

        print!("{}", table);
        assert_eq!(table, Board::new(2));
    }
}
