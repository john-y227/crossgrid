use std::env;
use std::fs;
use std::process::ExitCode;

use crossgrid::{number_cells, word_slots, Direction, Grid};

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let path = match args.next() {
        Some(path) => path,
        None => {
            eprintln!("usage: crossgrid <grid-file>");
            return ExitCode::FAILURE;
        }
    };

    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(err) => {
            eprintln!("could not read {path}: {err}");
            return ExitCode::FAILURE;
        }
    };

    let grid = match Grid::parse(&text) {
        Ok(grid) => grid,
        Err(err) => {
            eprintln!("could not parse {path}: {err}");
            return ExitCode::FAILURE;
        }
    };

    print_report(&grid);
    ExitCode::SUCCESS
}

fn print_report(grid: &Grid) {
    let slots = word_slots(grid);
    let across = slots.iter().filter(|s| s.direction == Direction::Across).count();
    let down = slots.iter().filter(|s| s.direction == Direction::Down).count();

    println!("size: {}x{}", grid.width(), grid.height());
    println!("black squares: {} ({:.1}%)", grid.black_count(), grid.black_ratio() * 100.0);
    println!("180-degree symmetry: {}", grid.has_180_symmetry());
    println!("numbered cells: {}", number_cells(grid).len());
    println!("slots: {across} across, {down} down");
}
