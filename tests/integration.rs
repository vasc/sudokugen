use sudoku_generator::board::Board;
use sudoku_generator::solver::solve;

#[test]
fn solve_sudoku() {
    let table: Board =
        "...4..87.4.3......2....3..9..62....7...9.6...3.9.8...........4.8725........72.6..".into();

    assert_eq!(
        solve(&table).unwrap(),
        "695412873413879526287653419146235987728946135359187264561398742872564391934721658".into()
    );
}

#[test]
fn solve_sudoku2() {
    let table: Board =
        ".724..3........49.........2921...5.7..4.6...3......2...4..7.....3..196....5..4.21".into();

    assert_eq!(
        solve(&table).unwrap(),
        "572491386318726495469583172921348567754962813683157249146275938237819654895634721".into()
    );
}

#[test]
fn test_solved() {
    let board = Board::from(
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
    ",
    );

    let solved = solve(&board).unwrap();

    assert_eq!(
        solved,
        Board::from(
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
        )
    );
}
