use sudokugen::{Board, Puzzle};

#[test]
fn solve_sudoku_simple() {
    let mut table: Board =
        "...4..87.4.3......2....3..9..62....7...9.6...3.9.8...........4.8725........72.6.."
            .parse()
            .unwrap();

    table.solve().unwrap();

    assert_eq!(
        table,
        "695412873413879526287653419146235987728946135359187264561398742872564391934721658"
            .parse()
            .unwrap()
    );
}

#[test]
fn solve_sudoku_with_backtrack() {
    let mut table: Board =
        ".724..3........49.........2921...5.7..4.6...3......2...4..7.....3..196....5..4.21"
            .parse()
            .unwrap();

    table.solve().unwrap();
    assert_eq!(
        table,
        "572491386318726495469583172921348567754962813683157249146275938237819654895634721"
            .parse()
            .unwrap()
    );
}

#[test]
fn test_solved() {
    let mut board: Board = "
    1 2 3 4 5 6 7 8 9
    1 2 3 4 5 6 7 8 9
    1 2 3 4 5 6 7 8 9
    1 2 3 4 5 6 7 8 9
    1 2 3 4 5 6 7 8 9
    1 2 3 4 5 6 7 8 9
    1 2 3 4 5 6 7 8 9
    1 2 3 4 5 6 7 8 9
    1 2 3 4 5 6 7 8 9
    "
    .parse()
    .unwrap();

    board.solve().unwrap();

    assert_eq!(
        board,
        "
    1 2 3 4 5 6 7 8 9
    1 2 3 4 5 6 7 8 9
    1 2 3 4 5 6 7 8 9
    1 2 3 4 5 6 7 8 9
    1 2 3 4 5 6 7 8 9
    1 2 3 4 5 6 7 8 9
    1 2 3 4 5 6 7 8 9
    1 2 3 4 5 6 7 8 9
    1 2 3 4 5 6 7 8 9
    "
        .parse()
        .unwrap()
    );
}

#[test]
fn generate_test() {
    let puzzle = Puzzle::generate(sudokugen::board::BoardSize::NineByNine);
    let board = puzzle.board();

    println!(
        "Final board ({})\n{}",
        board
            .iter_cells()
            .filter(|cell| board.get(cell).is_some())
            .count(),
        board,
    );

    assert!(puzzle.is_solution_unique());
}
