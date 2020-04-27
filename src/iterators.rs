// use crate::board::CellLoc;

// #[derive(Debug, Clone, Copy)]
// struct IterCells {
//     last_cell: Cell,
// }

// #[derive(Debug, Clone, Copy)]
// struct Cell {
//     base_size: usize,
//     index: usize,
// }
// impl CellLoc for Cell {
//     fn get_index(&self) -> usize {
//         self.index
//     }
// }

// impl Iterator for IterCells {
//     type Item = Cell;
//     fn next(&mut self) -> Option<Self::Item> {
//         if self.last_cell.index >= self.last_cell.base_size * self.last_cell.base_size {
//             None
//         } else {
//             self.last_cell.index += 1;
//             Some(self.last_cell)
//         }
//     }
// }
