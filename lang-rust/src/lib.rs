use ra_ide_api::{Analysis, AnalysisChange, AnalysisHost, FileId, HighlightedRange, SourceRootId};
use ra_syntax::TextRange;
use relative_path::RelativePathBuf;
use types::{
    BackBuffer, Buffer, BufferIndex, Client, ClientIndex, Cmd, DeleteDirection, Direction,
    GlobalData, JumpType, KeyData, Mode, Msg, Point, Rect, SecondaryMap, Utils,
};

mod colors;
use colors::{get_color_from_severity, get_color_from_tag};

#[derive(Debug, Default)]
pub struct Cursor {
    pub position: Point,
    pub stored_x: u16,
}

#[derive(Debug)]
struct State {
    analysisHost: AnalysisHost,
}

fn file_id_from_buffer_index(buffer_index: BufferIndex) -> FileId {
    FileId(KeyData::from(buffer_index).as_ffi() as u32)
}

impl State {
    fn new(global_data: &GlobalData) -> State {
        let mut analysisHost = AnalysisHost::default();
        let mut add_root = AnalysisChange::new();
        add_root.add_root(SourceRootId(0), true);
        analysisHost.apply_change(add_root);
        for buffer_index in global_data.buffer_keys.keys() {
            let buffer = &global_data.buffers[buffer_index];
            analysisHost.apply_change({
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
        State { analysisHost }
    }
}

pub fn get_pos_len_from_text_range(
    text_range: TextRange,
    buffer: &Buffer,
) -> (Option<Point>, usize) {
    let start_char_index = text_range.start().to_usize();
    let line_index = buffer.rope.char_to_line(start_char_index);
    let line_start_index = buffer.rope.line_to_char(line_index);
    let length = text_range.len().to_usize();
    let start_point = if line_index < buffer.start_line {
        None
    } else {
        Some(Point {
            x: (start_char_index - line_start_index + 4) as u16,
            y: (line_index - buffer.start_line) as u16,
        })
    };
    (start_point, length)
}

pub fn draw_diagnostics(
    analysis: &Analysis,
    file_id: FileId,
    buffer: &Buffer,
    back_buffer: &mut BackBuffer,
    utils: &Utils,
) {
    if let Ok(diagnostics) = analysis.diagnostics(file_id) {
        for diagnostic in diagnostics {
            (utils.info)(&format!("diagnostic: {:?}", diagnostic));
            if let (Some(start_point), length) =
                get_pos_len_from_text_range(diagnostic.range, buffer)
            {
                let color = get_color_from_severity(diagnostic.severity);
                (utils.style_range)(
                    back_buffer,
                    &start_point,
                    length,
                    None,
                    None,
                    Some(color.clone()),
                );
                let diagnostic_text_x =
                    buffer.rope.line(start_point.y as usize).len_chars() as u16 + 5;
                (utils.write_to_buffer)(
                    back_buffer,
                    &Point {
                        x: diagnostic_text_x,
                        y: start_point.y,
                    },
                    &diagnostic.message,
                    None,
                    None,
                    Some(color),
                );
            }
        }
    }
}

fn draw_syntax_highlighting(
    analysis: &Analysis,
    file_id: FileId,
    buffer: &Buffer,
    back_buffer: &mut BackBuffer,
    utils: &Utils,
    client: &Client,
) {
    if let Ok(highlighted_ranges) = analysis.highlight(file_id) {
        for HighlightedRange { range, tag, .. } in highlighted_ranges {
            (utils.info)(&format!("{:?} -> {}", range, tag));
            if let (Some(start_point), length) = get_pos_len_from_text_range(range, buffer) {
                (utils.info)(&format!("start point: {:?}", start_point));
                if start_point.y > client.size.as_ref().map(|s| s.h).unwrap_or(0) {
                    break;
                }
                let fg_color = get_color_from_tag(tag);
                if fg_color.is_some() {
                    (utils.style_range)(back_buffer, &start_point, length, None, fg_color, None);
                }
            }
        }
    }
}

#[no_mangle]
pub fn render(
    global_data: &GlobalData,
    client_index: &ClientIndex,
    back_buffer: &mut BackBuffer,
    utils: &Utils,
    data_ptr: *mut c_void,
) {
    let data: Box<State> = unsafe { Box::from_raw(data_ptr as *mut State) };
    let analysis = data.analysisHost.analysis();
    let client = &global_data.clients[*client_index];
    let buffer = &global_data.buffers[client.buffer];
    let file_id = file_id_from_buffer_index(client.buffer);
    draw_syntax_highlighting(&analysis, file_id, &buffer, back_buffer, utils, client);
    draw_diagnostics(&analysis, file_id, &buffer, back_buffer, utils);
    std::mem::forget(data);
}

#[no_mangle]
pub fn update(
    global_data: &mut GlobalData,
    msg: &Msg,
    utils: &Utils,
    _send_cmd: &Box<Fn(ClientIndex, Cmd)>,
    data_ptr: *mut c_void,
) {
    let mut data: Box<State> = unsafe { Box::from_raw(data_ptr as *mut State) };
    use Cmd::*;
    match msg {
        Msg::Cmd(client_index, cmd) => match cmd {
            BufferLoaded => {
                let buffer_index = global_data.clients[*client_index].buffer;
                let buffer = &mut global_data.buffers[buffer_index];
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
            BufferModified => {
                let buffer_index = global_data.clients[*client_index].buffer;
                let buffer = &mut global_data.buffers[buffer_index];
                data.analysisHost.apply_change({
                    let mut change = AnalysisChange::new();
                    let new_source = std::sync::Arc::new(String::from(buffer.rope.clone()));
                    (utils.info)(&format!("{}", new_source));
                    change.change_file(file_id_from_buffer_index(buffer_index), new_source);
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
pub fn init(global_data: &GlobalData) -> *mut c_void {
    Box::into_raw(Box::new(State::new(global_data))) as *mut c_void
}

#[no_mangle]
pub fn cleanup(data: *mut c_void) {
    unsafe {
        let ptr = Box::from_raw(data);
        drop(ptr);
    }
}
