use crate::back_buffer::write_to_buffer;
use types::Utils;

pub fn build_utils() -> Utils {
    Utils { write_to_buffer }
}
