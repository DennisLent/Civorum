use map::MapSize;

pub struct CliOptions {
    pub gui: bool,
    pub size: MapSize,
}


pub fn parse_cli() -> Result<CliOptions, String> {
    let mut gui = false;
    let mut size: Option<MapSize> = None;

    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--gui" | "-g" => gui = true,
            "--size" | "-s" => {
                let value = args.next().ok_or_else(|| {
                    format!(
                        "Expected a map size after '{}'. Available options: {}.",
                        arg,
                        MapSize::NAMES.join(", ")
                    )
                })?;
                size = Some(parse_size(&value)?);
            }
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            _ => {
                if let Some(value) = arg.strip_prefix("--size=") {
                    size = Some(parse_size(value)?);
                } else {
                    return Err(format!(
                        "Unknown argument '{}'. Use --help to see supported options.",
                        arg
                    ));
                }
            }
        }
    }

    Ok(CliOptions {
        gui,
        size: size.unwrap_or(MapSize::Standard),
    })
}

fn parse_size(value: &str) -> Result<MapSize, String> {
    value.parse::<MapSize>().map_err(|_| {
        format!(
            "Unknown map size '{}'. Available options: {}.",
            value,
            MapSize::NAMES.join(", ")
        )
    })
}

fn print_usage() {
    println!(
        "Usage: cargo run [--gui] [--size <{}>]",
        MapSize::NAMES.join("|")
    );
    println!("\nExamples:");
    println!("  cargo run -- --size standard");
    println!("  cargo run -- --gui --size huge");
}