use std::{env, io, path::PathBuf};

use civorum_core::render_debug_map;
use civorum_mapgen::pipeline::map_sizes::MapSizes;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args
        .get(1)
        .map(|v| v == "--help" || v == "-h")
        .unwrap_or(false)
    {
        print_usage();
        return Ok(());
    }

    let size = args
        .get(1)
        .map(String::as_str)
        .map(parse_size)
        .transpose()
        .map_err(invalid_input)?
        .unwrap_or(MapSizes::Standard);

    let seed = args
        .get(2)
        .map(String::as_str)
        .map(parse_seed)
        .transpose()
        .map_err(invalid_input)?
        .unwrap_or(Some(12));

    let cell_px = args
        .get(3)
        .map(String::as_str)
        .map(parse_cell_px)
        .transpose()
        .map_err(invalid_input)?
        .unwrap_or(16);

    let out_path = args
        .get(4)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("out/debug_map.png"));

    render_debug_map(seed, size, cell_px, &out_path)?;
    println!("Wrote {}", out_path.display());

    Ok(())
}

fn print_usage() {
    println!("Usage:");
    println!(
        "  cargo run -p civorum-core --bin render_debug_map -- [size] [seed|none] [cell_px] [out_path]"
    );
    println!("Defaults:");
    println!("  size=standard seed=12 cell_px=16 out_path=out/debug_map.png");
    println!("Sizes:");
    println!("  duel tiny small standard large huge");
}

fn parse_size(value: &str) -> Result<MapSizes, String> {
    match value.to_ascii_lowercase().as_str() {
        "duel" => Ok(MapSizes::Duel),
        "tiny" => Ok(MapSizes::Tiny),
        "small" => Ok(MapSizes::Small),
        "standard" => Ok(MapSizes::Standard),
        "large" => Ok(MapSizes::Large),
        "huge" => Ok(MapSizes::Huge),
        _ => Err(format!(
            "invalid size '{value}'. Use one of: duel, tiny, small, standard, large, huge"
        )),
    }
}

fn parse_seed(value: &str) -> Result<Option<u64>, String> {
    if value.eq_ignore_ascii_case("none") {
        return Ok(None);
    }

    value
        .parse::<u64>()
        .map(Some)
        .map_err(|_| format!("invalid seed '{value}'. Use an unsigned integer or 'none'"))
}

fn parse_cell_px(value: &str) -> Result<u32, String> {
    let parsed = value
        .parse::<u32>()
        .map_err(|_| format!("invalid cell_px '{value}'. Use an integer >= 10"))?;

    if parsed < 10 {
        return Err(format!(
            "invalid cell_px '{value}'. It must be >= 10 for hill visibility"
        ));
    }

    Ok(parsed)
}

fn invalid_input(message: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}
