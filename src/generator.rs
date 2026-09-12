use std::fmt;

use crate::grid::{Cell, Grid};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenerateError {
    InvalidDimensions,
    InvalidRatio,
}

impl fmt::Display for GenerateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenerateError::InvalidDimensions => write!(f, "width and height must both be at least 1"),
            GenerateError::InvalidRatio => write!(f, "black_ratio must be between 0.0 and 1.0"),
        }
    }
}

/// A minimal SplitMix64 generator. Its only job is to make `generate` a
/// pure function of its seed - same inputs, same layout, every time -
/// without pulling in a `rand` dependency for what is a handful of coin
/// flips.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Rng {
        Rng(seed)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A value in `[0.0, 1.0)`.
    fn next_unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// Generates a grid with 180-degree rotational symmetry whose black-square
/// density is approximately `black_ratio` (must be in `[0.0, 1.0]`). The
/// `seed` picks which layout to produce among all symmetric layouts at that
/// density; the same width, height, ratio, and seed always produce the same
/// grid.
///
/// Black squares are chosen in mirrored pairs, so the result always
/// satisfies `Grid::has_180_symmetry`. Afterward, any white cell that
/// happened to end up boxed in by black squares (or the grid edge) on all
/// four sides is converted to black too, since such a cell could never
/// belong to any word - leaving it white would just be a trap for whoever
/// fills in the grid next.
pub fn generate(width: usize, height: usize, black_ratio: f64, seed: u64) -> Result<Grid, GenerateError> {
    if width == 0 || height == 0 {
        return Err(GenerateError::InvalidDimensions);
    }
    if !(0.0..=1.0).contains(&black_ratio) {
        return Err(GenerateError::InvalidRatio);
    }

    let mut cells = vec![Cell::White(None); width * height];
    let mut rng = Rng::new(seed);

    for row in 0..height {
        for col in 0..width {
            let idx = row * width + col;
            let mirror = width * height - 1 - idx;
            if idx > mirror {
                continue;
            }
            if rng.next_unit() < black_ratio {
                cells[idx] = Cell::Black;
                cells[mirror] = Cell::Black;
            }
        }
    }

    for row in 0..height {
        for col in 0..width {
            let idx = row * width + col;
            if matches!(cells[idx], Cell::White(_)) && is_isolated(&cells, width, height, row, col) {
                let mirror = width * height - 1 - idx;
                cells[idx] = Cell::Black;
                cells[mirror] = Cell::Black;
            }
        }
    }

    let rows: Vec<Vec<Cell>> = cells.chunks(width).map(|chunk| chunk.to_vec()).collect();
    Ok(Grid::from_rows(&rows).expect("generated rows are always rectangular and non-empty"))
}

/// True if every side of `(row, col)` is either the grid edge or a black
/// square, meaning the cell could never be part of an across or down word.
fn is_isolated(cells: &[Cell], width: usize, height: usize, row: usize, col: usize) -> bool {
    let is_black = |r: usize, c: usize| matches!(cells[r * width + c], Cell::Black);
    let left_blocked = col == 0 || is_black(row, col - 1);
    let right_blocked = col + 1 == width || is_black(row, col + 1);
    let up_blocked = row == 0 || is_black(row - 1, col);
    let down_blocked = row + 1 == height || is_black(row + 1, col);
    left_blocked && right_blocked && up_blocked && down_blocked
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::numbering::{word_slots, Direction};

    #[test]
    fn rejects_zero_dimensions() {
        assert_eq!(generate(0, 5, 0.2, 1).unwrap_err(), GenerateError::InvalidDimensions);
        assert_eq!(generate(5, 0, 0.2, 1).unwrap_err(), GenerateError::InvalidDimensions);
    }

    #[test]
    fn rejects_out_of_range_ratio() {
        assert_eq!(generate(5, 5, -0.1, 1).unwrap_err(), GenerateError::InvalidRatio);
        assert_eq!(generate(5, 5, 1.1, 1).unwrap_err(), GenerateError::InvalidRatio);
    }

    #[test]
    fn same_seed_produces_same_layout() {
        let a = generate(9, 9, 0.2, 42).unwrap();
        let b = generate(9, 9, 0.2, 42).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn different_seeds_can_differ() {
        let a = generate(11, 11, 0.25, 1).unwrap();
        let b = generate(11, 11, 0.25, 2).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn generated_layouts_are_symmetric() {
        for seed in 0..5 {
            let grid = generate(13, 11, 0.18, seed).unwrap();
            assert!(grid.has_180_symmetry());
        }
    }

    #[test]
    fn zero_ratio_yields_no_black_squares() {
        let grid = generate(6, 6, 0.0, 7).unwrap();
        assert_eq!(grid.black_count(), 0);
    }

    #[test]
    fn full_ratio_yields_all_black_squares() {
        let grid = generate(6, 6, 1.0, 7).unwrap();
        assert_eq!(grid.black_count(), 36);
    }

    #[test]
    fn every_white_cell_belongs_to_a_word() {
        for seed in 0..8 {
            let grid = generate(15, 15, 0.22, seed).unwrap();
            let mut covered = HashSet::new();
            for slot in word_slots(&grid) {
                for i in 0..slot.length {
                    let cell = match slot.direction {
                        Direction::Across => (slot.row, slot.col + i),
                        Direction::Down => (slot.row + i, slot.col),
                    };
                    covered.insert(cell);
                }
            }
            for row in 0..grid.height() {
                for col in 0..grid.width() {
                    if !grid.is_black(row, col) {
                        assert!(covered.contains(&(row, col)), "cell ({row}, {col}) is not part of any word");
                    }
                }
            }
        }
    }
}
