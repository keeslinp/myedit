mod back_buffer;
mod editor;
mod send_cmd;
mod utils;

use structopt::StructOpt;
#[derive(Debug, StructOpt)]
#[structopt(
    name = "myedit",
    about = "Personalized text editor and dev environment"
)]
struct Opt {
    #[structopt(name = "command", long = "command")]
    sub_command: Option<String>,
    #[structopt(name = "target", long = "target")]
    target: Option<String>,
    #[structopt(parse(from_os_str))]
    input: Option<std::path::PathBuf>,
}

#[derive(StructOpt, Debug)]
enum SubCommand {
    #[structopt(name = "--edit")]
    Edit(EditCommand),
}

#[derive(StructOpt, Debug)]
struct EditCommand {
    #[structopt(parse(from_os_str))]
    input: std::path::PathBuf,
}

fn main() {
    let opt = Opt::from_args();
    match opt.sub_command {
        Some(command) => {
            if let Some(target) = opt.target {
                send_cmd::send(target.as_str(), command.as_str());
            } else {
                panic!("Cannot pass command without target");
            }
        }
        None => editor::start(opt.input),
    }
}
