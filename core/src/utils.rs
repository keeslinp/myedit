use crate::back_buffer::write_to_buffer;
use types::Utils;
use log::{info, debug, warn};

fn info(msg: String) {
    info!("{}", msg);
}

fn warn(msg: String) {
    warn!("{}", msg);
}

fn debug(msg: String) {
    debug!("{}", msg);
}

pub fn build_utils() -> Utils {
    Utils { write_to_buffer, info, warn, debug }
}
