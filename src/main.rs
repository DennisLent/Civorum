use map::worldgen::{feature_scale_for_dims, random_map, realistic_map, MapSize};
use map::Map;
use std::env;

#[cfg(feature = "gui")]
mod gui;

#[derive(Copy, Clone, Debug)]
enum Mode { Realistic, Random }

#[derive(Clone, Debug)]
struct Args {
    gui: bool,
    seed: u32,
    sea_level: f32,
    mode: Mode,
    width: Option<i32>,
    height: Option<i32>,
    size: Option<String>,
    feature_scale: Option<f32>,
}

fn parse_args() -> Args {
    let mut a = Args {
        gui: false,
        seed: 42,
        sea_level: 0.0,
        mode: Mode::Realistic,
        width: None,
        height: None,
        size: Some("standard".into()),
        feature_scale: None,
    };
    let mut it = env::args().skip(1).peekable();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--gui" => a.gui = true,
            "--seed" => if let Some(v) = it.next() { a.seed = v.parse().unwrap_or(a.seed) },
            "--width" => if let Some(v) = it.next() { a.width = v.parse().ok() },
            "--height" => if let Some(v) = it.next() { a.height = v.parse().ok() },
            "--size" => if let Some(v) = it.next() { a.size = Some(v.to_lowercase()) },
            "--sea-level" => if let Some(v) = it.next() { a.sea_level = v.parse().unwrap_or(a.sea_level) },
            "--mode" => if let Some(v) = it.next() {
                a.mode = match v.to_lowercase().as_str() {"random"=>Mode::Random, _=>Mode::Realistic};
            },
            "--feature-scale" => if let Some(v) = it.next() { a.feature_scale = v.parse().ok() },
            "-h"|"--help" => { print_help(); std::process::exit(0); }
            _ => {}
        }
    }
    a
}

fn print_help() {
    eprintln!("Usage: civorum [--gui] [--seed N] [--width W --height H | --size NAME] [--sea-level F] [--mode realistic|random] [--feature-scale F]\n\nSizes: duel,tiny,small,standard,large,huge");
}

fn size_to_dims(name: &str) -> (i32, i32) {
    match name {
        "duel" => MapSize::Duel.dims(),
        "tiny" => MapSize::Tiny.dims(),
        "small" => MapSize::Small.dims(),
        "standard" => MapSize::Standard.dims(),
        "large" => MapSize::Large.dims(),
        "huge" => MapSize::Huge.dims(),
        _ => MapSize::Standard.dims(),
    }
}

fn build_map(args: &Args) -> Map {
    let (width, height) = match (args.width, args.height) {
        (Some(w), Some(h)) => (w, h),
        _ => args.size.as_deref().map(size_to_dims).unwrap_or(MapSize::Standard.dims()),
    };
    match args.mode {
        Mode::Realistic => realistic_map(width, height, args.seed, args.sea_level),
        Mode::Random => {
            let scale = args.feature_scale.unwrap_or_else(|| feature_scale_for_dims(width, height));
            random_map(width, height, args.seed, args.sea_level, scale)
        }
    }
}

fn summarize_map(m: &Map) {
    use map::Terrain;
    let mut land = 0usize; let mut water = 0usize;
    let mut biome_counts: std::collections::BTreeMap<String, usize> = Default::default();
    let mut water_counts: std::collections::BTreeMap<String, usize> = Default::default();
    for (_, t) in m.iter() {
        match t.terrain {
            Terrain::Land => { land += 1; *biome_counts.entry(format!("{:?}", t.biome.unwrap())).or_insert(0usize) += 1; }
            Terrain::Water => { water += 1; *water_counts.entry(format!("{:?}", t.water.unwrap())).or_insert(0usize) += 1; }
        }
    }
    println!("Tiles: {} (land {} / water {})", m.len(), land, water);
    println!("Biomes:");
    for (b, c) in biome_counts { println!("  {}: {}", b, c); }
    println!("Water:");
    for (d, c) in water_counts { println!("  {}: {}", d, c); }
}

fn main() {
    let args = parse_args();
    let map = build_map(&args);
    if args.gui {
        #[cfg(feature = "gui")]
        {
            gui::run(map);
            return;
        }
        #[cfg(not(feature = "gui"))]
        eprintln!("This binary was built without GUI support. Rebuild with --features gui.");
    } else {
        summarize_map(&map);
    }
}
