mod viewer;
mod cli;

use cli::parse_cli;

fn main() {
    let options = parse_cli().unwrap_or_else(|err| {
        eprintln!("{}", err);
        std::process::exit(1);
    });

    if options.gui {
        viewer::run_gui(options.size);
    }
}




