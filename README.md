# Sudokugen

Sudoku puzzle solver and generator library.

Sudokugen can find a solution to a valid puzzle using a mixture of basic strategies
and bruteforce. It can also generate new minimal puzzles.
This library was built as a rust learning project for myself.

## How to use Sudokugen
Sudokugen offers two convenience functions, [`solve`] and [`generate`] to solve and generate
sudoku puzzles and a struct [`Board`] to help you inspect and manipulate them.

You can parse a puzzle from a string:

```rust
use sudokugen::Board;

let board: Board = "
      . . . | 4 . . | 8 7 .
      4 . 3 | . . . | . . .
      2 . . | . . 3 | . . 9
      ---------------------
      . . 6 | 2 . . | . . 7
      . . . | 9 . 6 | . . .
      3 . 9 | . 8 . | . . .
      ---------------------
      . . . | . . . | . 4 .
      8 7 2 | 5 . . | . . .
      . . . | 7 2 . | 6 . .
".parse().unwrap();
```

After it's parsed you can solve it using the [`solve`] function:
```rust
use sudokugen::solve;

assert_eq!(
     solve(&board).unwrap(),
     "695412873413879526287653419146235987728946135359187264561398742872564391934721658"
     .parse()
     .unwrap()
 );
```

Finally you can generate new puzzles using [`generate`], the parameter `3` here indicates that
you would like a puzzle of base size 3, which translates to a 9x9 puzzle.

```rust
use sudokugen::generate;

let puzzle = generate(3);

println!("Puzzle\n{}", puzzle.board());
println!("Solution\n{}", puzzle.solution());
```

Which will print something like this:

 ```
 > Puzzle
 > . . . . . . . 6 .
 > . 1 7 . 4 . . 9 .
 > . . . . 9 . 5 3 .
 > . . 5 . 7 2 8 . .
 > 1 . . . . 8 4 5 .
 > . 4 . 9 . . . . .
 > 8 7 9 1 2 . . . .
 > 4 5 . 8 . . . . .
 > . . . . . . . . .
 >
 > Solution
 > 9 2 3 5 8 1 7 6 4
 > 5 1 7 6 4 3 2 9 8
 > 6 8 4 2 9 7 5 3 1
 > 3 6 5 4 7 2 8 1 9
 > 1 9 2 3 6 8 4 5 7
 > 7 4 8 9 1 5 6 2 3
 > 8 7 9 1 2 6 3 4 5
 > 4 5 6 8 3 9 1 7 2
 > 2 3 1 7 5 4 9 8 6
 ```

## Crate Layout
This crate is divided in three modules. [`board`] contains the tools needed to parse, manipulate and print
a puzzle and it's individual cells. [`solver`] contains the [`solve`] function and [`generator`] contains
the [`generate`] function as well as an umbrella struct to hold the puzzle and it's solution.

## Puzzle quality
Grading puzzles is beyond the scope of this crate. The reason behind it is that grading puzzles
correctly, requires solving them like a human would and some of the more complex techniques to solve
a puzzle like a human would require a lot of computations that do not always payoff performance-wise.

That being said, the generated puzzles consistently have between 22 and 26 clues making them likely
on the harder side of most generally available puzzles.

## Is it fast?
The quick answer is, it depends on your use case. The [`solve`] function is optimized to be
decently fast for a 9x9 sudoku puzzle, in my 2017 MacBook Pro it takes an average of 300μs
to solve a difficult puzzle, that is around 3000 puzzles per second.

The [`generate`] function is less optimized and makes heavy usage of [`solve`] without trying to
re-use repeated computations, as such it's much slower clocking at about 18ms to generate
a new puzzle in my benchmarks.

You can run your own benchmarks with `cargo bench`

[`solve`]: https://docs.rs/sudokugen/0.1.0/sudokugen/solver/fn.solve.html
[`solver`]: https://docs.rs/sudokugen/0.1.0/sudokugen/solver/index.html
[`generate`]: https://docs.rs/sudokugen/0.1.0/sudokugen/solver/generator/fn.generate.html
[`generator`]: https://docs.rs/sudokugen/0.1.0/sudokugen/solver/generator/index.html
[`Board`]: https://docs.rs/sudokugen/0.1.0/sudokugen/board/struct.Board.html
[`board`]: https://docs.rs/sudokugen/0.1.0/sudokugen/board/index.html