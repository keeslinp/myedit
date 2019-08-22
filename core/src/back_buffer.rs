use log::warn;
use std::io::Write;
use types::{BackBuffer, Cell, Color, Point, Rect, Style};
use ropey::RopeSlice;

pub fn index_from_point(back_buffer: &BackBuffer, p: &Point) -> usize {
    (p.y * back_buffer.dim.w + p.x) as usize
}

fn apply_updates_to_cell(
    cell: &mut Cell,
    letter: Option<char>,
    style: Option<Style>,
    fg: Option<Color>,
    bg: Option<Color>,
) {
    if letter.is_some() {
        cell.value = letter;
    }
    if style.is_some() {
        cell.style = style;
    }
    if fg.is_some() {
        cell.fg = fg;
    }
    if bg.is_some() {
        cell.bg = bg;
    }
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
        if offset + index > back_buffer.cells.len() {
            warn!("overflow");
            break;
        }
        let cell = &mut back_buffer.cells[index + offset];
        apply_updates_to_cell(cell, None, style.clone(), fg.clone(), bg.clone());
    }
}

pub fn style_rope_slice_range(
    back_buffer: &mut BackBuffer,
    rope_slice: &RopeSlice,
    mut position: Point,
    style: Option<Style>,
    fg: Option<Color>,
    bg: Option<Color>,
) {
    for line in rope_slice.lines() {
        style_range(back_buffer, &position, line.len_chars(), style.clone(), fg.clone(), bg.clone());
        position.x = 4;
        position.y += 1;
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
            apply_updates_to_cell(
                &mut back_buffer.cells[index],
                Some(c),
                style.clone(),
                fg.clone(),
                bg.clone(),
            );
            p.x += 1;
        }
    }
}

pub fn update_stdout(old_buffer: &BackBuffer, new_buffer: &BackBuffer, out: impl Write) {
    use termion::{
        cursor::{Goto, Restore, Save, Show},
        style::Reset,
    };
    let mut writer = out; //HideCursor::from(out);
    let mut x = 1;
    let mut y = 1;
    write!(writer, "{}", Save).unwrap();
    for (old_cell, new_cell) in old_buffer.cells.iter().zip(new_buffer.cells.iter()) {
        if old_cell != new_cell {
            write!(writer, "{}{}", Goto(x, y), Reset).unwrap();
            if let Some(ref fg) = new_cell.fg {
                write!(writer, "\x1b[38;2;{};{};{}m", fg.r, fg.g, fg.b).unwrap();
            }
            if let Some(ref bg) = new_cell.bg {
                write!(writer, "\x1b[48;2;{};{};{}m", bg.r, bg.g, bg.b).unwrap();
            }
            if let Some(c) = new_cell.value {
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

pub fn create_back_buffer(size: Rect) -> BackBuffer {
    // TODO: Figure out the client size
    let Rect { w, h } = size;
    let total_cell_count = w * h;
    let mut cells = Vec::with_capacity(total_cell_count as usize);
    for _ in 0..total_cell_count {
        cells.push(Cell::default());
    }
    BackBuffer { cells, dim: size }
}
