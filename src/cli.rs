use map::MapSize;

pub struct CliOptions {
    pub gui: bool,
    pub size: MapSize,
    pub seed: u64,
}


pub fn parse_cli() -> Result<CliOptions, String> {
    let mut gui = false;
    let mut size: Option<MapSize> = None;
    let mut seed: Option<u64> = None;

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
            "--seed" | "-S" => {
                let value = args.next().ok_or_else(|| {
                    format!("Expected a numeric seed after '{}'.", arg)
                })?;
                seed = Some(parse_seed(&value)?);
            }
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            _ => {
                if let Some(value) = arg.strip_prefix("--size=") {
                    size = Some(parse_size(value)?);
                } else if let Some(value) = arg.strip_prefix("--seed=") {
                    seed = Some(parse_seed(value)?);
                } else {
                    return Err(format!(
                        "Unknown argument '{}'. Use --help to see supported options.",
                        arg
                    ));
                }
            }
        }
    }

    // If no seed provided, derive a random-ish seed from system time and pid
    let seed = seed.unwrap_or_else(random_seed);

    Ok(CliOptions {
        gui,
        size: size.unwrap_or(MapSize::Standard),
        seed,
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

fn parse_seed(value: &str) -> Result<u64, String> {
    value
        .parse::<u64>()
        .map_err(|_| format!("Invalid seed '{}': expected an unsigned integer", value))
}

fn random_seed() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let nanos = now.as_nanos() as u64;
    let pid = std::process::id() as u64;
    // Simple mix
    let mut z = nanos ^ (pid.wrapping_mul(0x9E37_79B9_7F4A_7C15));
    z ^= z >> 33;
    z = z.wrapping_mul(0xff51_afd7_ed55_8ccd);
    z ^= z >> 33;
    z = z.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    z ^= z >> 33;
    z
}

fn print_usage() {
    println!(
        "Usage: cargo run [--gui] [--size <{}>] [--seed <u64>]",
        MapSize::NAMES.join("|")
    );
    println!("\nExamples:");
    println!("  cargo run -- --size standard");
    println!("  cargo run -- --gui --size huge");
    println!("  cargo run -- --gui --seed 123456789");
}
