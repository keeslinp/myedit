use crate::back_buffer::{style_range, style_rope_slice_range, write_to_buffer};
use log::{debug, info, warn};
use types::Utils;

fn info(msg: &str) {
    info!("{}", msg);
}

fn warn(msg: &str) {
    warn!("{}", msg);
}

fn debug(msg: &str) {
    debug!("{}", msg);
}

pub fn build_utils() -> Utils {
    Utils {
        write_to_buffer,
        info,
        warn,
        debug,
        style_range,
        style_rope_slice_range,
    }
}
