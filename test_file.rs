use std::any::Any; test
use termion::{color, style};
use types::{GlobalData, Msg};

struct LocalData {}
test
#[no_mangle]
pub fn render(global_data: &GlobalData) {
    print!(
        "{}{}{}Stuff",
        style::Reset,
        termion::clear::All,
        termion::cursor::Goto(1, 1)
    );
    println!("{}Yellodw", color::Fg(color::Yellow));
    println!("{}Blue", color::Fg(color::White));
    println!("Blue'n'Bold{}", color::Fg(color::Reset));
    println!("{}Just plain italic{}", style::Invert, style::Reset);
}

#[no_mangle]
pub fn update(global_data: &GlobalData, msg: &Msg) {}

#[no_mangle]
pub fn init() -> Box<Any> {
    Box::new(LocalData {})
}
testuse std::any::Any; test
testuse termion::{color, style};
use types::{GlobalData, Msg};

struct LocalData {}
test
#[no_mangle]
pub fn render(global_data: &GlobalData) {
    print!(
        "{}{}{}Stuff",
        style::Reset,
        termion::clear::All,
        termion::cursor::Goto(1, 1)
    );
    println!("{}Yellodw", color::Fg(color::Yellow));
    println!("{}Blue", color::Fg(color::White));
    println!("Blue'n'Bold{}", color::Fg(color::Reset));
    println!("{}Just plain italic{}", style::Invert, style::Reset);
}

#[no_mangle]
pub fn update(global_data: &GlobalData, msg: &Msg) {}

#[no_mangle]
pub fn init() -> Box<Any> {
    Box::new(LocalData {})
}
testuse std::any::Any; test
testuse termion::{color, style};
use types::{GlobalData, Msg};

struct LocalData {}
test
#[no_mangle]
pub fn render(global_data: &GlobalData) {
    print!(
        "{}{}{}Stuff",
        style::Reset,
        termion::clear::All,
        termion::cursor::Goto(1, 1)
    );
    println!("{}Yellodw", color::Fg(color::Yellow));
    println!("{}Blue", color::Fg(color::White));
    println!("Blue'n'Bold{}", color::Fg(color::Reset));
    println!("{}Just plain italic{}", style::Invert, style::Reset);
}

#[no_mangle]
pub fn update(global_data: &GlobalData, msg: &Msg) {}

#[no_mangle]
pub fn init() -> Box<Any> {
    Box::new(LocalData {})
}
testuse std::any::Any; test
testuse termion::{color, style};
use types::{GlobalData, Msg};

struct LocalData {}
test
#[no_mangle]
pub fn render(global_data: &GlobalData) {
    print!(
        "{}{}{}Stuff",
        style::Reset,
        termion::clear::All,
        termion::cursor::Goto(1, 1)
    );
    println!("{}Yellodw", color::Fg(color::Yellow));
    println!("{}Blue", color::Fg(color::White));
    println!("Blue'n'Bold{}", color::Fg(color::Reset));
    println!("{}Just plain italic{}", style::Invert, style::Reset);
}

#[no_mangle]
pub fn update(global_data: &GlobalData, msg: &Msg) {}

#[no_mangle]
pub fn init() -> Box<Any> {
    Box::new(LocalData {})
}
