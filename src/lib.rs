//! Data structures and pure functions for reasoning about the *shape* of a
//! crossword grid: where the black squares sit, whether the layout numbers
//! correctly, what word slots it produces, and what letters (if any) fill
//! the white cells. This crate does not solve puzzles or interpret clue
//! text - it only describes the grid, carrying clues through as opaque
//! strings where a file format (like `.puz`) happens to bundle them in.

mod generator;
mod grid;
mod numbering;
mod puz;

pub use generator::{generate, GenerateError};
pub use grid::{Cell, Grid, GridError};
pub use numbering::{number_cells, word_slots, Direction, Slot};
pub use puz::{read_puz, PuzError, PuzFile};
