use std::ffi::c_void;
use std::sync::mpsc::Sender;
use syntect::{
    easy::HighlightLines,
    highlighting::{Style, ThemeSet},
    parsing::SyntaxSet,
};
use types::{BackBuffer, GlobalData, Msg, Point, Utils, Color};

#[derive(Debug)]
struct Data {
    ps: SyntaxSet,
    ts: ThemeSet,
}

impl Default for Data {
    fn default() -> Data {
        Data {
            ps: SyntaxSet::load_defaults_newlines(),
            ts: ThemeSet::load_defaults(),
        }
    }
}

fn color_from_syntect_color(color: &syntect::highlighting::Color) -> Color {
    Color {
        r: color.r,
        g: color.g,
        b: color.b,
    }
}

#[no_mangle]
pub fn render(global_data: &GlobalData, back_buffer: &mut BackBuffer, utils: &Utils, data_ptr: *mut c_void) {
    let data: Box<Data> = unsafe { Box::from_raw(data_ptr as *mut Data) };
    let buffer = &global_data.buffers[global_data.current_buffer];
    let syntax = global_data.buffers[global_data.current_buffer].source.extension().and_then(|ext| ext.to_str()).and_then(|extension| data.ps.find_syntax_by_extension(extension)).expect("loading syntax style");
    let mut h = HighlightLines::new(syntax, &data.ts.themes["Solarized (dark)"]);
    let (cols, rows) = termion::terminal_size().unwrap();
    for (index, ranges) in buffer
        .rope
        .lines()
        .skip(buffer.start_line)
        .take(rows as usize - 1)
        .map(|line| h.highlight(line.as_str().unwrap_or(""), &data.ps))
        .enumerate()
    {
        let mut x_pos = 0;
        for (style, text) in ranges {
            (utils.write_to_buffer)(
                back_buffer,
                &Point {
                    x: x_pos,
                    y: index as u16,
                },
                text,
                None,
                Some(color_from_syntect_color(&style.foreground)),
                None,
                // Some(color_from_syntect_color(&style.background)),
            );
            x_pos += text.len() as u16;
        }
    }
    std::mem::forget(data);
}

#[no_mangle]
pub fn update(_global_data: &GlobalData, _msg: &Msg, _utils: &Utils, _msg_sender: &Sender<Msg>) {}

#[no_mangle]
pub fn init() -> *mut c_void {
    unsafe { Box::into_raw(Box::new(Data::default())) as *mut c_void }
}

#[no_mangle]
pub fn cleanup(data: *mut c_void) {
    unsafe {
        let ptr = Box::from_raw(data);
        drop(ptr);
    }
}
