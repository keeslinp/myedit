use ra_ide_api::{Analysis, AnalysisChange, AnalysisHost, FileId, HighlightedRange, SourceRootId};
use types::{
    BackBuffer, BufferIndex, ClientIndex, Cmd, DeleteDirection, Direction, GlobalData, JumpType,
    KeyData, Mode, Msg, Point, Rect, SecondaryMap, Utils,
};
mod colors;
use colors::get_color_from_tag;

#[derive(Debug, Default)]
pub struct Cursor {
    pub position: Point,
    pub stored_x: u16,
}

#[derive(Debug)]
struct State {
    analysisHost: AnalysisHost,
}

impl Default for State {
    fn default() -> State {
        let mut analysisHost = AnalysisHost::default();
        let mut add_root = AnalysisChange::new();
        add_root.add_root(SourceRootId(0), true);
        analysisHost.apply_change(add_root);
        State { analysisHost }
    }
}

fn file_id_from_buffer_index(buffer_index: BufferIndex) -> FileId {
    FileId(KeyData::from(buffer_index).as_ffi() as u32)
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
    let analysis = data.analysisHost.analysis();
    let client = &global_data.clients[*client_index];
    let file_id = file_id_from_buffer_index(client.buffer);
    if let Ok(highlighted_ranges) = analysis.highlight(file_id) {
        for HighlightedRange { range, tag, .. } in highlighted_ranges {
            // (utils.info)(&format!("{:?} -> {}", range, tag));
            let start_char_index = range.start().to_usize();
            let buffer = &global_data.buffers[client.buffer];
            let line_index = buffer.rope.char_to_line(start_char_index);
            if line_index < buffer.start_line {
                continue;
            }
            if line_index
                > buffer.start_line + client.size.as_ref().map(|s| s.h).unwrap_or(0) as usize
            {
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
                let buffer_index = global_data.clients[*client_index].buffer;
                let buffer = &mut global_data.buffers[buffer_index];
                use relative_path::RelativePathBuf;
                data.analysisHost.apply_change({
                    let mut change = AnalysisChange::new();
                    change.add_file(
                        SourceRootId(0),
                        file_id_from_buffer_index(buffer_index),
                        RelativePathBuf::from_path(buffer.source.as_path())
                            .expect("building relative path"),
                        std::sync::Arc::new(String::from(buffer.rope.clone())),
                    );
                    change
                });
            }
            InsertChar(_) | DeleteChar(_) => {
                let buffer_index = global_data.clients[*client_index].buffer;
                let buffer = &mut global_data.buffers[buffer_index];
                data.analysisHost.apply_change({
                    let mut change = AnalysisChange::new();
                    change.change_file(
                        file_id_from_buffer_index(buffer_index),
                        std::sync::Arc::new(String::from(buffer.rope.clone())),
                    );
                    change
                });
            }
            _ => {}
        },
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
