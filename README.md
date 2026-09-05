# crossgrid

A small Rust library for reasoning about the *shape* of a crossword
grid - where the black squares go, whether the layout has the usual
180-degree rotational symmetry, how the cells get numbered, and what
across/down word slots the grid produces. It does not know about letters,
clues, or solving a puzzle. That keeps every public function pure: give it
a grid, get back an answer, no state and no I/O involved.

Constructing a crossword grid by hand is fiddly - a single stray black
square breaks the symmetry, or turns a 5-letter slot into a 3-letter one
and a disconnected 1-letter one nobody notices until the grid is half
filled with clues. This crate is meant to be the checking layer underneath
an editor or generator: build a candidate layout, ask this library whether
it is well-formed, and only then move on to filling it with words.

## Grid format

A grid is a block of text, one line per row. `#` is a black square;
everything else on the line is a white square (so `.` is the usual
choice, but any placeholder character works):

```
#...#
..#..
.....
..#..
#...#
```

## Library usage

```rust
use crossgrid::{Grid, word_slots, Direction};

let grid = Grid::parse("...\n.#.\n...").expect("valid grid");

assert_eq!(grid.width(), 3);
assert!(grid.has_180_symmetry());

let across_words = word_slots(&grid)
    .into_iter()
    .filter(|slot| slot.direction == Direction::Across)
    .count();
assert_eq!(across_words, 2);
```

## CLI usage

The `crossgrid` binary reads a grid file and prints a short report:

```
$ cargo run -- puzzle.txt
size: 5x5
black squares: 5 (20.0%)
180-degree symmetry: true
numbered cells: 8
slots: 5 across, 5 down
```

## Design

- `Grid` is an immutable value built by `Grid::parse` or `Grid::from_rows`.
  There is no in-place mutation, so a `Grid` can be freely shared and
  compared.
- `number_cells` and `word_slots` are plain functions of a `&Grid` with no
  side effects, which makes them straightforward to test with small
  hand-written ASCII grids (see the `#[cfg(test)]` modules in `src/`).
- Zero third-party dependencies. Standard library only.

## Status

Early skeleton: grid parsing, symmetry checking, numbering, and slot
extraction are implemented and tested. Letter content, grid generation,
and file formats beyond the plain-text one are not yet built.
