use crate::board::Board;
use rand::prelude::*;
// use std::collections::HashMap;
// use std::num;

pub fn gen() -> Board {
    let mut board = Board::new(3);
    let mut rng = thread_rng();
    // let mut cell = self.cells.choose_mut(&mut rng).unwrap();
    // cell.value = Some(rng.gen_range(1, 10));

    for l in 0..9 {
        for c in 0..9 {
            board.set_line_mut(l, c, rng.gen_range(1, 10));
        }
    }

    board
}

pub fn merge_boards(board_a: &Board, board_b: &Board) -> Board {
    let mut board = Board::new(3);
    let mut rng = thread_rng();
    let probability = board_a.fitness() as f64 / (board_a.fitness() + board_b.fitness()) as f64;

    for l in 0..9 {
        for c in 0..9 {
            let value = if rng.gen_bool(0.01) {
                Some(rng.gen_range(1, 10))
            } else if rng.gen_bool(probability) {
                board_a.get_at(l, c)
            } else {
                board_b.get_at(l, c)
            };

            board.set_line_mut(l, c, value.unwrap());
        }
    }

    board
}
