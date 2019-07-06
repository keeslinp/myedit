use termion::{color, style};

#[crate_type="dylib"]

#[no_mangle]
pub fn render() {
  print!("{}{}{}Stuff", style::Reset, termion::clear::All, termion::cursor::Goto(1, 1));
  println!("{}Yellow", color::Fg(color::Yellow));
  println!("{}Blue", color::Fg(color::Blue));
  println!("{}Blue'n'Bold{}", style::Bold, style::Reset);
  println!("{}Blue'n'Bold{} 2", style::Bold, style::Reset);
  println!("{}Just plain italic", style::Italic);
}
