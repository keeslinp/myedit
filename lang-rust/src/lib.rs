use types::{
    BackBuffer, Cmd, DeleteDirection, Direction, GlobalData, JumpType, Mode, Msg, Point, Utils, ClientIndex, SecondaryMap, BufferIndex, Rect,
};
use ra_ide_api::{AnalysisHost, AnalysisChange, Analysis, FileId, HighlightedRange};
mod colors;
use colors::get_color_from_tag;

#[derive(Debug, Default)]
pub struct Cursor {
    pub position: Point,
    pub stored_x: u16,
}

#[derive(Debug, Default)]
struct State {
    analysis: Option<(Analysis, FileId)>,
}

#[no_mangle]
pub fn render(
    global_data: &GlobalData,
    client_index: &ClientIndex,
    back_buffer: &mut BackBuffer,
    utils: &Utils,
    data_ptr: *mut c_void,
) {
    let mut data: Box<State> = unsafe { Box::from_raw(data_ptr as *mut State) };
    if let Some((ref analysis, file_id)) = data.analysis {
        if let Ok(highlighted_ranges) = analysis.highlight(file_id) {
            for HighlightedRange { range, tag, .. } in highlighted_ranges {
                (utils.info)(&format!("{:?} -> {}", range, tag));
                let client = &global_data.clients[*client_index];
                let buffer = &global_data.buffers[client.buffer];
                let start_char_index = range.start().to_usize();
                let line_index = buffer.rope.char_to_line(start_char_index);
                if line_index < buffer.start_line {
                    continue;
                }
                if line_index > buffer.start_line + client.size.as_ref().map(|s| s.h).unwrap_or(0) as usize {
                    break;
                }
                let fg_color = get_color_from_tag(tag);
                if fg_color.is_some() {
                    let line_start_index = buffer.rope.line_to_char(line_index);
                    let start_point = Point {
                        x: (start_char_index - line_start_index + 4) as u16,
                        y: (line_index - buffer.start_line) as u16,
                    };
                    let length = range.len().to_usize();
                    (utils.style_range)(back_buffer, &start_point, length, None, fg_color, None);
                }
            }
        }
    } else {
        (utils.info)("No analysis set");
    }
    std::mem::forget(data);
}

#[no_mangle]
pub fn update(
    global_data: &mut GlobalData,
    msg: &Msg,
    utils: &Utils,
    send_cmd: &Box<Fn(ClientIndex, Cmd)>,
    data_ptr: *mut c_void,
) {
    let mut data: Box<State> = unsafe { Box::from_raw(data_ptr as *mut State) };
    use Cmd::*;
    match msg {
        Msg::Cmd(client_index, cmd) => match cmd {
            BufferLoaded => {
                let buffer = &mut global_data.buffers[global_data.clients[*client_index].buffer];
                let string_rope = String::from(buffer.rope.clone());
                (utils.info)(&string_rope);
                data.analysis = Some(Analysis::from_single_file(string_rope));
                (utils.info)("Analysis loaded");
            },
            _ => {},
        }
        _ => {}
    };
    std::mem::forget(data);
}

use std::ffi::c_void;

#[no_mangle]
pub fn init() -> *mut c_void {
    Box::into_raw(Box::new(State::default())) as *mut c_void
}

#[no_mangle]
pub fn cleanup(data: *mut c_void) {
    unsafe {
        let ptr = Box::from_raw(data);
        drop(ptr);
    }
}
