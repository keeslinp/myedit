use std::{time, thread, fs};

fn main() {
  let one_second = time::Duration::from_millis(1000);
  unsafe {
    loop {
      fs::copy("../target/debug/librender.dylib", "./libs/librender.dylib").expect("copying render lib");
      let render_lib = libloading::Library::new("./libs/librender.dylib").expect("failed to load");
      let func: libloading::Symbol<unsafe extern fn()> = render_lib.get(b"render").expect("loading render function");
      func();
      thread::sleep(one_second);
    }
  };
}
