# crossgrid

A small Rust library for reasoning about the *shape* of a crossword
grid - where the black squares go, whether the layout has the usual
180-degree rotational symmetry, how the cells get numbered, what
across/down word slots the grid produces, and what letters (if any) fill
the white cells. It does not know about clues or solving a puzzle. That
keeps every public function pure: give it a grid, get back an answer, no
state and no I/O involved.

Constructing a crossword grid by hand is fiddly - a single stray black
square breaks the symmetry, or turns a 5-letter slot into a 3-letter one
and a disconnected 1-letter one nobody notices until the grid is half
filled with clues. This crate is meant to be the checking layer underneath
an editor or generator: build a candidate layout, ask this library whether
it is well-formed, and only then move on to filling it with words.

## Grid format

A grid is a block of text, one line per row. `#` is a black square; an
ASCII letter is a filled white square holding that letter; anything else
is an empty white square (`.` is the usual choice, but any placeholder
character works):

```
#...#
..#..
.....
..#..
#...#
```

A filled puzzle uses the same format, with letters in place of the
placeholder character:

```
#CAT#
..O..
DOG..
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
filled cells: 0 of 20 white
180-degree symmetry: true
numbered cells: 8
slots: 5 across, 5 down
```

## Generating layouts

`generate` builds a symmetric layout from a target size and black-square
density instead of requiring one to be typed out by hand. It takes a seed
so the result is reproducible - the same width, height, ratio, and seed
always produce the same grid:

```rust
use crossgrid::generate;

let grid = generate(15, 15, 0.16, 42).expect("valid parameters");
assert!(grid.has_180_symmetry());
```

The generator never leaves behind a white cell that is boxed in on all
four sides, since a cell like that could never be part of any word.

## Design

- `Grid` is an immutable value built by `Grid::parse` or `Grid::from_rows`.
  There is no in-place mutation, so a `Grid` can be freely shared and
  compared.
- `number_cells` and `word_slots` are plain functions of a `&Grid` with no
  side effects, which makes them straightforward to test with small
  hand-written ASCII grids (see the `#[cfg(test)]` modules in `src/`).
- Zero third-party dependencies. Standard library only.

## Status

Early skeleton: grid parsing, symmetry checking, numbering, slot
extraction, letter content, and symmetric layout generation are
implemented and tested. File formats beyond the plain-text one are not
yet built.
