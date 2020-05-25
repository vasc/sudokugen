pub mod board;
pub mod solver;
use clap::{App, Arg, SubCommand};

// use solver::generate;
use sudokugen::generate;

fn main() {
    let matches = App::new("SudokuGen")
        .version("0.1.0")
        .about("Solve and generate sudoku puzzles in pure rust")
        .subcommand(
            SubCommand::with_name("gen")
                .about("Generate sudoku puzzles")
                .arg(Arg::with_name("INPUT").index(1)),
        )
        .subcommand(SubCommand::with_name("solve").about("Solve a sudoku puzzle"))
        .get_matches();

    if let Some(_) = matches.subcommand_matches("gen") {
        let puzzle = generate(3);
        // let board = res.board();

        println!("Puzzle\n{}", puzzle.board());
        println!("Solution\n{}", puzzle.solution());
    }
}
