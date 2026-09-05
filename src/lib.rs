//! Data structures and pure functions for reasoning about the *shape* of a
//! crossword grid: where the black squares sit, whether the layout numbers
//! correctly, and what word slots it produces. This crate does not know
//! about letters, clues, or solving a puzzle - it only describes the grid.

mod grid;
mod numbering;

pub use grid::{Cell, Grid, GridError};
pub use numbering::{number_cells, word_slots, Direction, Slot};
