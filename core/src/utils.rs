use crate::back_buffer::{write_to_buffer, style_range};
use types::Utils;
use log::{info, debug, warn};

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
    Utils { write_to_buffer, info, warn, debug, style_range }
}
