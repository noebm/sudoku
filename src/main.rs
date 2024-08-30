use std::ops::Deref;

// Cells are either a value or possible values!

#[derive(Copy, Clone)]
enum Cell<S, T> {
    Value(S),
    Possibly(T),
}

impl<S, T> Cell<S, T> {
    fn finished(&self) -> bool {
        matches!(self, Cell::Value(..))
    }

    fn get(&self) -> Option<&S> {
        if let Cell::Value(value) = self {
            Some(value)
        } else {
            None
        }
    }
}

#[derive(Clone)]
struct SudokuCell(Cell<u8, Vec<u8>>);

impl SudokuCell {
    fn all() -> Vec<u8> {
        (1..=9).collect()
    }

    fn eliminate(&mut self, value: u8) {
        if let Cell::Possibly(values) = &mut self.0 {
            // FIXME this only works for elements contained at most 1 time!
            values
                .iter()
                .position(|&x| x == value)
                .map(|index| values.remove(index));

            assert!(!values.is_empty());

            if values.len() == 1 {
                self.0 = Cell::Value(values[0])
            }
        }
    }
}

impl Deref for SudokuCell {
    type Target = Cell<u8, Vec<u8>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

struct Sudoku {
    board: Vec<SudokuCell>,
}

impl TryFrom<String> for Sudoku {
    type Error = ();
    fn try_from(mut value: String) -> Result<Self, Self::Error> {
        value = value.lines().map(|line| line.trim()).collect();

        if value.len() != 81 {
            return Err(());
        }

        let mut board = Sudoku::new();

        for (idx, c) in value.chars().enumerate() {
            if let '1'..='9' = c {
                board.board[idx] = SudokuCell(Cell::Value(c.to_string().parse().map_err(|_| ())?));
            }
        }
        Ok(board)
    }
}

impl std::fmt::Display for Sudoku {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (idx, x) in self.board.iter().enumerate() {
            if idx % 9 == 0 && idx != 0 {
                writeln!(f)?;
            }
            let value = x.get().unwrap_or(&0);
            write!(f, "{value}")?;
        }
        Ok(())
    }
}

impl Sudoku {
    fn new() -> Self {
        let cell = SudokuCell(Cell::Possibly(SudokuCell::all()));
        Self {
            board: vec![cell.clone(); 81],
        }
    }

    fn finished(&self) -> bool {
        self.board.iter().all(|x| x.finished())
    }

    fn solve_constraint(&mut self, indices: [usize; 9]) {
        let values: Vec<_> = indices
            .iter()
            .filter_map(|&idx| self.board[idx].get())
            .copied()
            .collect();

        for value in values {
            for idx in indices {
                self.board[idx].eliminate(value)
            }
        }
    }

    fn index(row: usize, column: usize) -> usize {
        row * 9 + column
    }

    fn row_indices(row: usize) -> [usize; 9] {
        (0..9)
            .map(|column| Self::index(row, column))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }

    fn column_indices(column: usize) -> [usize; 9] {
        (0..9)
            .map(|row| Self::index(row, column))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }

    fn box_indices(box_index: usize) -> [usize; 9] {
        let base_row = box_index % 3;
        let base_column = box_index / 3;
        let mut indices = [0; 9];
        let mut i = 0;
        for row in 0..3 {
            for column in 0..3 {
                indices[i] = Self::index(base_row * 3 + row, base_column * 3 + column);
                i += 1;
            }
        }

        indices
    }

    fn solve_rows(&mut self) {
        for indices in (0..9).map(Self::row_indices) {
            self.solve_constraint(indices);
        }
    }

    fn solve_columns(&mut self) {
        for indices in (0..9).map(Self::column_indices) {
            self.solve_constraint(indices);
        }
    }

    fn solve_boxes(&mut self) {
        for indices in (0..9).map(Self::box_indices) {
            self.solve_constraint(indices);
        }
    }

    fn step(&mut self) {
        self.solve_rows();
        self.solve_columns();
        self.solve_boxes();
    }

    fn solve(&mut self) {
        let mut step_count = 0;
        while !self.finished() {
            self.step();
            step_count += 1;
            if step_count > 81 {
                println!("Could not solve board. Aborting!");
                break;
            }
        }
    }
}

const BOARD: &str = "
003020600
900305001
001806400
008102900
700000008
006708200
002609500
800203009
005010300
";

fn main() {
    let mut sudoku = Sudoku::try_from(BOARD.to_string()).unwrap();
    println!("{sudoku}\n");

    sudoku.solve();
    println!("{sudoku}\n");
}
