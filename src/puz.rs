use std::fmt;

use crate::grid::{Cell, Grid};

const MAGIC: &[u8; 12] = b"ACROSS&DOWN\0";
const HEADER_LEN: usize = 0x34;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PuzError {
    /// Shorter than a bare header, let alone a board.
    TooShort,
    /// Missing the `ACROSS&DOWN\0` signature at offset 0x02.
    BadMagic,
    /// The header promises a board that the rest of the file doesn't hold.
    Truncated,
    /// Width or height is zero.
    InvalidDimensions,
}

impl fmt::Display for PuzError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PuzError::TooShort => write!(f, "file is too short to contain a .puz header"),
            PuzError::BadMagic => write!(f, "missing ACROSS&DOWN file signature"),
            PuzError::Truncated => write!(f, "file ends before the board it declares"),
            PuzError::InvalidDimensions => write!(f, "width or height is zero"),
        }
    }
}

/// The parts of a `.puz` file this crate can make sense of.
///
/// This only reads the base header, the two boards, and the string table
/// (title, author, copyright, clues, notes). Checksums are not verified,
/// since a mismatch there doesn't change what grid the file describes.
/// Extra sections used for rebus squares, timers, and circled-square
/// markup (`GRBS`/`RTBL`/`LTIM`/`GEXT`) are not parsed - a rebus cell in
/// either board decodes as an empty white square rather than the rebus
/// fragment it stands for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PuzFile {
    pub title: String,
    pub author: String,
    pub copyright: String,
    pub notes: String,
    pub clues: Vec<String>,
    pub solution: Grid,
    pub player_state: Grid,
}

/// Parses the bytes of a `.puz` file.
pub fn read_puz(bytes: &[u8]) -> Result<PuzFile, PuzError> {
    if bytes.len() < HEADER_LEN {
        return Err(PuzError::TooShort);
    }
    if &bytes[0x02..0x0E] != MAGIC.as_slice() {
        return Err(PuzError::BadMagic);
    }

    let width = bytes[0x2C] as usize;
    let height = bytes[0x2D] as usize;
    if width == 0 || height == 0 {
        return Err(PuzError::InvalidDimensions);
    }
    let num_clues = u16::from_le_bytes([bytes[0x2E], bytes[0x2F]]) as usize;

    let board_len = width * height;
    let solution_end = HEADER_LEN + board_len;
    let state_end = solution_end + board_len;
    if bytes.len() < state_end {
        return Err(PuzError::Truncated);
    }

    let solution = board_to_grid(&bytes[HEADER_LEN..solution_end], width, false);
    let player_state = board_to_grid(&bytes[solution_end..state_end], width, true);

    // Title, author, copyright, one string per clue, then notes - in that
    // fixed order, each terminated by a single 0x00 byte. A file that ends
    // early (no notes section, say) just yields empty strings for whatever
    // is missing rather than an error.
    let mut strings = bytes[state_end..].split(|&b| b == 0).map(latin1_decode);
    let mut next_string = move || strings.next().unwrap_or_default();

    let title = next_string();
    let author = next_string();
    let copyright = next_string();
    let clues = (0..num_clues).map(|_| next_string()).collect();
    let notes = next_string();

    Ok(PuzFile { title, author, copyright, notes, clues, solution, player_state })
}

/// Converts one `width`-wide block of board bytes into a `Grid`. `is_state`
/// selects how a fill byte is read: the player-state board uses `-` for an
/// unfilled white square, a marker the solution board never contains.
fn board_to_grid(bytes: &[u8], width: usize, is_state: bool) -> Grid {
    let rows: Vec<Vec<Cell>> = bytes
        .chunks(width)
        .map(|row| {
            row.iter()
                .map(|&b| match b {
                    b'.' => Cell::Black,
                    b'-' if is_state => Cell::White(None),
                    b if b.is_ascii_alphabetic() => Cell::White(Some(b.to_ascii_uppercase() as char)),
                    _ => Cell::White(None),
                })
                .collect()
        })
        .collect();
    Grid::from_rows(&rows).expect("board_to_grid always produces height rows of width columns")
}

/// `.puz` strings are single-byte Latin-1, not UTF-8, so every byte value
/// decodes to a valid character and this can never fail.
fn latin1_decode(bytes: &[u8]) -> String {
    bytes.iter().map(|&b| b as char).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_puz() -> Vec<u8> {
        let mut bytes = vec![0u8; HEADER_LEN];
        bytes[0x02..0x0E].copy_from_slice(MAGIC);
        bytes[0x18..0x1C].copy_from_slice(b"1.3\0");
        bytes[0x2C] = 3; // width
        bytes[0x2D] = 1; // height
        bytes[0x2E..0x30].copy_from_slice(&1u16.to_le_bytes()); // num_clues
        bytes.extend_from_slice(b"CAT"); // solution
        bytes.extend_from_slice(b"CA-"); // player state: T not yet filled in
        bytes.push(0); // title
        bytes.push(0); // author
        bytes.push(0); // copyright
        bytes.extend_from_slice(b"a clue");
        bytes.push(0);
        bytes.push(0); // notes
        bytes
    }

    #[test]
    fn reads_minimal_puz_file() {
        let puz = read_puz(&minimal_puz()).unwrap();
        assert_eq!(puz.title, "");
        assert_eq!(puz.author, "");
        assert_eq!(puz.copyright, "");
        assert_eq!(puz.notes, "");
        assert_eq!(puz.clues, vec!["a clue".to_string()]);
        assert_eq!(puz.solution, Grid::parse("CAT").unwrap());
        assert_eq!(puz.player_state, Grid::parse("CA-").unwrap());
    }

    #[test]
    fn rejects_short_files() {
        assert_eq!(read_puz(&[0u8; 10]).unwrap_err(), PuzError::TooShort);
    }

    #[test]
    fn rejects_bad_magic() {
        let bytes = vec![0u8; HEADER_LEN];
        assert_eq!(read_puz(&bytes).unwrap_err(), PuzError::BadMagic);
    }

    #[test]
    fn rejects_zero_dimensions() {
        let mut bytes = vec![0u8; HEADER_LEN];
        bytes[0x02..0x0E].copy_from_slice(MAGIC);
        assert_eq!(read_puz(&bytes).unwrap_err(), PuzError::InvalidDimensions);
    }

    #[test]
    fn rejects_truncated_board() {
        let mut bytes = vec![0u8; HEADER_LEN];
        bytes[0x02..0x0E].copy_from_slice(MAGIC);
        bytes[0x2C] = 5;
        bytes[0x2D] = 5;
        assert_eq!(read_puz(&bytes).unwrap_err(), PuzError::Truncated);
    }
}
