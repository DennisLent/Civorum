mod viewer;
mod cli;
mod camera;

use cli::parse_cli;
use map::Map;

fn main() {
    let options = parse_cli().unwrap_or_else(|err| {
        eprintln!("{}", err);
        std::process::exit(1);
    });

    let map = Map::new(options.size);

    if options.gui {
        viewer::run_gui(map, options.seed);
    } else {
        println!("map size: {}", map.size());
        println!("map width: {}", map.width());
        println!("map height: {}", map.height());
    }
}

