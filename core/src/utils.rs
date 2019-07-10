use types::{ Utils, BackBuffer, Point, Color, Style, Cell };
use crate::back_buffer::write_to_buffer;

pub fn build_utils() -> Utils {
    Utils {
        write_to_buffer,
    }
}
