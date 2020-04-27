pub mod board;
pub mod gen;
pub mod iterators;

// use board::Board;
// use gen::{gen, merge_boards};

// fn main() {
//     // let mut board = Board::new();
//     // println!("Table fitness: {}", board.fitness());

//     // let mut i = 0;
//     // loop {
//     //     if i > 100 {
//     //         break;
//     //     }
//     //     board.set_random();
//     //     board.print();
//     //     println!("Table fitness: {}", board.fitness());
//     //     i += 1;
//     // }

//     let mut boards: Vec<Board> = (0..100).map(|_| gen()).collect();

//     loop {
//         boards.sort_by_cached_key(|b| b.fitness());
//         for board in &boards {
//             println!("Board fitness: {}", board.fitness());
//         }

//         boards.truncate(50);
//         let mut new_boards = Vec::with_capacity(100);

//         for i in 0..100 {
//             let start = i / 2;
//             let end = start + 1;

//             if let Some(board_a) = boards.get(start) {
//                 if let Some(board_b) = boards.get(end) {
//                     new_boards.push(merge_boards(board_a, board_b))
//                 }
//             }
//         }
//         if new_boards.first().unwrap().fitness() == 0 {
//             break;
//         }
//         boards = new_boards;
//     }
// }
