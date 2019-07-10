use types::{BackBuffer, Cell, Color, Point, Rect, Style};
use std::io::Write;

pub fn index_from_point(back_buffer: &BackBuffer, p: &Point) -> usize {
    (p.y * back_buffer.dim.w + p.x) as usize
}

pub fn write_to_buffer(
    back_buffer: &mut BackBuffer,
    start_point: &Point,
    value: &str,
    style: Option<Style>,
    fg: Option<Color>,
    bg: Option<Color>,
) {
    // println!("({}, {}){}", start_point.x, start_point.y, value);
    let mut p = start_point.clone();
    for c in value.chars() {
        if c == '\n' {
            p.x = 0;
            p.y += 1;
        } else {
            let index = index_from_point(back_buffer, &p);
            back_buffer.cells[index] = Cell {
                value: Some(c),
                style: style.clone(),
                fg: fg.clone(),
                bg: bg.clone(),
            };
            p.x += 1;
        }
    }
}

pub fn update_stdout(new_buffer: &BackBuffer) {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    use termion::{ style::Reset, cursor::{HideCursor, Right, Goto}};
    let mut writer = HideCursor::from(handle);
    let mut x = 0;
    let mut y = 0;
    // return;
    write!(writer, "{}{}", Goto(1, 1), Reset);
    for cell in new_buffer.cells.iter() {
        if let Some(c) = cell.value {
            if c == '\n' {
            }
            write!(writer, "{}", c);
        } else {
            write!(writer, " ");
        }
        x += 1;
        if x >= new_buffer.dim.w {
            x = 0;
            y += 1;
            write!(writer, "{}", Goto(1, y + 1));
        }
    }
    writer.flush().unwrap();
}

pub fn create_back_buffer() -> BackBuffer {
    let (cols, rows) = termion::terminal_size().unwrap();
    let total_cell_count = cols * rows;
    let mut cells = Vec::with_capacity(total_cell_count as usize);
    for _ in 0..total_cell_count {
        cells.push(Cell::default());
    }
    BackBuffer {
        cells,
        dim: Rect { w: cols, h: rows },
    }
}
