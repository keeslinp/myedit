use std::io::Write;
use types::{BackBuffer, Cell, Color, Point, Rect, Style};

pub fn index_from_point(back_buffer: &BackBuffer, p: &Point) -> usize {
    (p.y * back_buffer.dim.w + p.x) as usize
}

pub fn style_range(
    back_buffer: &mut BackBuffer,
    start_point: &Point,
    length: usize,
    style: Option<Style>,
    fg: Option<Color>,
    bg: Option<Color>,
) {
    let index = index_from_point(back_buffer, start_point);
    for offset in 0..length {
        let cell = &mut back_buffer.cells[index + offset];
        if style.is_some() {
            cell.style = style.clone();
        }
        if fg.is_some() {
            cell.fg = fg.clone();
        }
        if bg.is_some() {
            cell.bg = bg.clone();
        }
    }
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
        if p.y >= back_buffer.dim.h {
            break;
        }
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

pub fn update_stdout(old_buffer: &BackBuffer, new_buffer: &BackBuffer) {
    let stdout = std::io::stdout();
    let handle = stdout.lock();
    use termion::{
        cursor::{Goto, HideCursor, Restore, Save, Show},
        style::Reset,
    };
    let mut writer = HideCursor::from(handle);
    let mut x = 1;
    let mut y = 2;
    write!(writer, "{}", Save).unwrap();
    for (old_cell, new_cell) in old_buffer.cells.iter().zip(new_buffer.cells.iter()) {
        if old_cell != new_cell {
            use termion::color;
            write!(writer, "{}{}", Goto(x, y), Reset).unwrap();
            if let Some(ref fg) = new_cell.fg {
                write!(writer, "\x1b[38;2;{};{};{}m", fg.r, fg.g, fg.b).unwrap();
            }
            if let Some(ref bg) = new_cell.bg {
                write!(writer, "\x1b[48;2;{};{};{}m", bg.r, bg.g, bg.b).unwrap();
            }
            if let Some(c) = new_cell.value {
                if c == '\n' {}
                write!(writer, "{}", c).unwrap();
            } else {
                write!(writer, " ").unwrap();
            }
        }
        x += 1;
        if x >= new_buffer.dim.w {
            x = 0;
            y += 1;
        }
    }
    write!(writer, "{}{}", Restore, Show).unwrap();
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
