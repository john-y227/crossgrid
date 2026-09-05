use std::collections::HashMap;

use crate::grid::Grid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Across,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slot {
    pub number: u32,
    pub direction: Direction,
    pub row: usize,
    pub col: usize,
    pub length: usize,
}

/// Assigns crossword numbers to white cells, following the standard
/// left-to-right, top-to-bottom convention: a cell is numbered if it starts
/// an across word or a down word (or both).
///
/// Returns `(row, col, number)` triples in numbering order.
pub fn number_cells(grid: &Grid) -> Vec<(usize, usize, u32)> {
    let mut numbered = Vec::new();
    let mut next_number = 1;
    for row in 0..grid.height() {
        for col in 0..grid.width() {
            if grid.is_black(row, col) {
                continue;
            }
            if starts_across(grid, row, col) || starts_down(grid, row, col) {
                numbered.push((row, col, next_number));
                next_number += 1;
            }
        }
    }
    numbered
}

/// Lists every across and down slot (a maximal run of two or more white
/// cells) together with the number assigned to its starting cell.
pub fn word_slots(grid: &Grid) -> Vec<Slot> {
    let numbers: HashMap<(usize, usize), u32> =
        number_cells(grid).into_iter().map(|(row, col, n)| ((row, col), n)).collect();

    let mut slots = Vec::new();

    for row in 0..grid.height() {
        for col in 0..grid.width() {
            if grid.is_black(row, col) || !starts_across(grid, row, col) {
                continue;
            }
            let length = (col..grid.width()).take_while(|&c| !grid.is_black(row, c)).count();
            slots.push(Slot {
                number: numbers[&(row, col)],
                direction: Direction::Across,
                row,
                col,
                length,
            });
        }
    }

    for col in 0..grid.width() {
        for row in 0..grid.height() {
            if grid.is_black(row, col) || !starts_down(grid, row, col) {
                continue;
            }
            let length = (row..grid.height()).take_while(|&r| !grid.is_black(r, col)).count();
            slots.push(Slot {
                number: numbers[&(row, col)],
                direction: Direction::Down,
                row,
                col,
                length,
            });
        }
    }

    slots
}

fn starts_across(grid: &Grid, row: usize, col: usize) -> bool {
    let open_start = col == 0 || grid.is_black(row, col - 1);
    let has_room = col + 1 < grid.width() && !grid.is_black(row, col + 1);
    open_start && has_room
}

fn starts_down(grid: &Grid, row: usize, col: usize) -> bool {
    let open_start = row == 0 || grid.is_black(row - 1, col);
    let has_room = row + 1 < grid.height() && !grid.is_black(row + 1, col);
    open_start && has_room
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Grid;

    #[test]
    fn numbers_a_simple_grid() {
        // 1 2
        // 3 .
        let grid = Grid::parse("..\n..").unwrap();
        let numbers = number_cells(&grid);
        assert_eq!(numbers, vec![(0, 0, 1), (0, 1, 2), (1, 0, 3)]);
    }

    #[test]
    fn single_cell_runs_are_not_slots() {
        let grid = Grid::parse("#.#").unwrap();
        assert!(word_slots(&grid).is_empty());
    }

    #[test]
    fn finds_across_and_down_slots() {
        let grid = Grid::parse("...\n.#.\n...").unwrap();
        let slots = word_slots(&grid);
        let across = slots.iter().filter(|s| s.direction == Direction::Across).count();
        let down = slots.iter().filter(|s| s.direction == Direction::Down).count();
        assert_eq!(across, 2);
        assert_eq!(down, 2);
    }

    #[test]
    fn slot_length_matches_run_of_white_cells() {
        let grid = Grid::parse("....").unwrap();
        let slots = word_slots(&grid);
        assert_eq!(slots.len(), 1);
        assert_eq!(slots[0].length, 4);
    }
}
