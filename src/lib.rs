//! Data structures and pure functions for reasoning about the *shape* of a
//! crossword grid: where the black squares sit, whether the layout numbers
//! correctly, what word slots it produces, and what letters (if any) fill
//! the white cells. This crate does not know about clues or solving a
//! puzzle - it only describes the grid.

mod grid;
mod numbering;

pub use grid::{Cell, Grid, GridError};
pub use numbering::{number_cells, word_slots, Direction, Slot};
