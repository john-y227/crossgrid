use std::fmt;

/// A single square in a crossword grid. A white square carries an optional
/// letter, so the same type represents both an empty layout and a filled
/// puzzle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Black,
    White(Option<char>),
}

/// A rectangular crossword grid: a fixed width/height array of cells.
///
/// Structural functions (symmetry, numbering, slots) only care whether a
/// cell is black or white, so they still work unchanged on a grid whose
/// white cells happen to carry letters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GridError {
    Empty,
    RaggedRows { row: usize, expected: usize, found: usize },
}

impl fmt::Display for GridError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GridError::Empty => write!(f, "grid has no rows"),
            GridError::RaggedRows { row, expected, found } => {
                write!(f, "row {row} has {found} cells, expected {expected}")
            }
        }
    }
}

impl Grid {
    /// Builds a grid from a rectangular block of cells. Fails if any row's
    /// length differs from the first row's length.
    pub fn from_rows(rows: &[Vec<Cell>]) -> Result<Grid, GridError> {
        let height = rows.len();
        if height == 0 {
            return Err(GridError::Empty);
        }
        let width = rows[0].len();
        for (i, row) in rows.iter().enumerate() {
            if row.len() != width {
                return Err(GridError::RaggedRows { row: i, expected: width, found: row.len() });
            }
        }
        let cells = rows.iter().flatten().copied().collect();
        Ok(Grid { width, height, cells })
    }

    /// Parses a text block into a grid. `#` marks a black square; an ASCII
    /// letter marks a filled white square holding that letter (case is
    /// normalized to uppercase); any other character marks an empty white
    /// square. Blank lines are skipped entirely rather than treated as
    /// zero-width rows, so trailing newlines in a file don't turn into a
    /// parse error.
    pub fn parse(text: &str) -> Result<Grid, GridError> {
        let rows: Vec<Vec<Cell>> = text
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| {
                line.chars()
                    .map(|c| match c {
                        '#' => Cell::Black,
                        c if c.is_ascii_alphabetic() => Cell::White(Some(c.to_ascii_uppercase())),
                        _ => Cell::White(None),
                    })
                    .collect()
            })
            .collect();
        Grid::from_rows(&rows)
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn get(&self, row: usize, col: usize) -> Option<Cell> {
        if row >= self.height || col >= self.width {
            return None;
        }
        self.cells.get(row * self.width + col).copied()
    }

    pub fn is_black(&self, row: usize, col: usize) -> bool {
        matches!(self.get(row, col), Some(Cell::Black))
    }

    /// The letter filling a white cell, or `None` for a black cell or an
    /// empty white cell.
    pub fn letter(&self, row: usize, col: usize) -> Option<char> {
        match self.get(row, col) {
            Some(Cell::White(letter)) => letter,
            _ => None,
        }
    }

    pub fn black_count(&self) -> usize {
        self.cells.iter().filter(|c| **c == Cell::Black).count()
    }

    /// Number of white cells that hold a letter.
    pub fn filled_count(&self) -> usize {
        self.cells.iter().filter(|c| matches!(c, Cell::White(Some(_)))).count()
    }

    /// Number of white cells, filled or not.
    pub fn white_count(&self) -> usize {
        self.cells.iter().filter(|c| matches!(c, Cell::White(_))).count()
    }

    /// Fraction of cells that are black, in `[0.0, 1.0]`.
    pub fn black_ratio(&self) -> f64 {
        if self.cells.is_empty() {
            return 0.0;
        }
        self.black_count() as f64 / self.cells.len() as f64
    }

    /// True if the grid reads the same when rotated 180 degrees around its
    /// center - the symmetry convention used by American-style crosswords.
    pub fn has_180_symmetry(&self) -> bool {
        for row in 0..self.height {
            for col in 0..self.width {
                let mirror_row = self.height - 1 - row;
                let mirror_col = self.width - 1 - col;
                if self.is_black(row, col) != self.is_black(mirror_row, mirror_col) {
                    return false;
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_grid() {
        let grid = Grid::parse("..#\n...\n#..").unwrap();
        assert_eq!(grid.width(), 3);
        assert_eq!(grid.height(), 3);
        assert!(grid.is_black(0, 2));
        assert!(!grid.is_black(1, 1));
    }

    #[test]
    fn rejects_ragged_rows() {
        let err = Grid::parse("..\n...").unwrap_err();
        assert_eq!(err, GridError::RaggedRows { row: 1, expected: 2, found: 3 });
    }

    #[test]
    fn rejects_empty_input() {
        assert_eq!(Grid::parse("").unwrap_err(), GridError::Empty);
    }

    #[test]
    fn black_ratio_counts_correctly() {
        let grid = Grid::parse("#.\n..").unwrap();
        assert_eq!(grid.black_count(), 1);
        assert!((grid.black_ratio() - 0.25).abs() < f64::EPSILON);
    }

    #[test]
    fn parses_letters_into_white_cells() {
        let grid = Grid::parse("CAT\n#a#\n...").unwrap();
        assert_eq!(grid.letter(0, 0), Some('C'));
        assert_eq!(grid.letter(0, 1), Some('A'));
        assert_eq!(grid.letter(1, 1), Some('A'));
        assert!(!grid.is_black(1, 1));
        assert_eq!(grid.letter(2, 0), None);
        assert_eq!(grid.letter(1, 0), None);
        assert!(grid.is_black(1, 0));
    }

    #[test]
    fn counts_filled_and_white_cells() {
        let grid = Grid::parse("CA#\n..#").unwrap();
        assert_eq!(grid.filled_count(), 2);
        assert_eq!(grid.white_count(), 4);
        assert_eq!(grid.black_count(), 2);
    }

    #[test]
    fn detects_180_symmetry() {
        let symmetric = Grid::parse("#..\n...\n..#").unwrap();
        assert!(symmetric.has_180_symmetry());

        let not_symmetric = Grid::parse("#..\n...\n...").unwrap();
        assert!(!not_symmetric.has_180_symmetry());
    }
}
