pub mod board;
use board::Board;

fn main() {
    let mut table = Board::new(3);

    table.solve();
    println!("{}", table);
}
