use std::{error::Error, fs::create_dir_all, path::Path};

use civorum_mapgen::map_components::terrain::Terrain;
use image::{ImageBuffer, Rgb};

const INV_SQRT3: f32 = 0.57735;
const BG_COLOR: Rgb<u8> = Rgb([20, 20, 20]);
const BORDER_COLOR: Rgb<u8> = Rgb([0, 0, 0]);

pub fn render_map_png(
    terrain: &[Terrain],
    hills: &[bool],
    width: i32,
    height: i32,
    cell_px: u32,
    out_path: &Path,
) -> Result<(), Box<dyn Error>> {
    if width <= 0 || height <= 0 {
        return Err("width and height must be positive".into());
    }
    if cell_px < 10 {
        return Err("cell_px must be >= 10 for hill marker visibility".into());
    }

    let width_u32 = u32::try_from(width)?;
    let height_u32 = u32::try_from(height)?;
    let expected_len = usize::try_from(width_u32)?
        .checked_mul(usize::try_from(height_u32)?)
        .ok_or("width * height overflow")?;

    if terrain.len() != expected_len || hills.len() != expected_len {
        return Err("terrain/hills length must match width * height".into());
    }

    let row_step = (cell_px * 3) / 4;
    let img_w = width_u32
        .checked_mul(cell_px)
        .and_then(|v| v.checked_add(cell_px / 2))
        .ok_or("image width overflow")?;
    let img_h = height_u32
        .checked_mul(row_step)
        .and_then(|v| v.checked_add(cell_px))
        .ok_or("image height overflow")?;

    let mut img = ImageBuffer::from_pixel(img_w, img_h, BG_COLOR);

    for y in 0..height_u32 {
        let row_x_offset = if y % 2 == 1 { cell_px / 2 } else { 0 };
        let oy = y * row_step;

        for x in 0..width_u32 {
            let ox = x * cell_px + row_x_offset;
            let idx = usize::try_from(y * width_u32 + x)?;
            let tile_terrain = terrain[idx];
            let base = terrain_color(tile_terrain);

            for py in 0..cell_px {
                for px in 0..cell_px {
                    if !inside_hex(px as i32, py as i32, cell_px) {
                        continue;
                    }

                    let gx = ox + px;
                    let gy = oy + py;
                    if gx >= img_w || gy >= img_h {
                        continue;
                    }

                    let border = is_border(px as i32, py as i32, cell_px);
                    let color = if border { BORDER_COLOR } else { base };
                    img.put_pixel(gx, gy, color);
                }
            }

            if hills[idx] && allows_hill_marker(tile_terrain) {
                draw_hill_marker(&mut img, ox, oy, cell_px, marker_color(base));
            }
        }
    }

    if let Some(parent) = out_path.parent() {
        if !parent.as_os_str().is_empty() {
            create_dir_all(parent)?;
        }
    }
    img.save(out_path)?;
    Ok(())
}

fn terrain_color(terrain: Terrain) -> Rgb<u8> {
    match terrain {
        Terrain::Grassland => Rgb([76, 175, 80]),
        Terrain::Plains => Rgb([183, 198, 90]),
        Terrain::Desert => Rgb([227, 197, 122]),
        Terrain::Tundra => Rgb([143, 168, 146]),
        Terrain::Snow => Rgb([242, 246, 248]),
        Terrain::CoastLake => Rgb([91, 183, 214]),
        Terrain::Ocean => Rgb([31, 95, 175]),
        Terrain::Mountain => Rgb([107, 107, 107]),
    }
}

fn allows_hill_marker(terrain: Terrain) -> bool {
    !matches!(
        terrain,
        Terrain::Ocean | Terrain::CoastLake | Terrain::Mountain
    )
}

fn marker_color(base: Rgb<u8>) -> Rgb<u8> {
    let [r, g, b] = base.0;
    let luminance = 0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32;
    if luminance < 120.0 {
        Rgb([230, 230, 230])
    } else {
        Rgb([30, 30, 30])
    }
}

fn inside_hex(px: i32, py: i32, cell_px: u32) -> bool {
    let r = cell_px as f32 / 2.0;
    let cx = r;
    let cy = r;
    let dx = (px as f32 - cx).abs();
    let dy = (py as f32 - cy).abs();

    dy <= r && (dx + dy * INV_SQRT3) <= r
}

fn is_border(px: i32, py: i32, cell_px: u32) -> bool {
    if !inside_hex(px, py, cell_px) {
        return false;
    }

    const DIRS: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    DIRS.iter()
        .any(|(dx, dy)| !inside_hex(px + dx, py + dy, cell_px))
}

fn draw_hill_marker(
    img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    ox: u32,
    oy: u32,
    cell_px: u32,
    color: Rgb<u8>,
) {
    let cx = (cell_px / 2) as i32;
    let cy = (cell_px / 2) as i32;
    let top = (cx, cy - (cell_px as i32 / 5));
    let left = (cx - (cell_px as i32 / 6), cy + (cell_px as i32 / 8));
    let right = (cx + (cell_px as i32 / 6), cy + (cell_px as i32 / 8));

    for py in 0..cell_px {
        for px in 0..cell_px {
            if !inside_hex(px as i32, py as i32, cell_px) {
                continue;
            }

            if !point_in_triangle((px as i32, py as i32), top, left, right) {
                continue;
            }

            let gx = ox + px;
            let gy = oy + py;
            if gx < img.width() && gy < img.height() {
                img.put_pixel(gx, gy, color);
            }
        }
    }
}

fn point_in_triangle(p: (i32, i32), a: (i32, i32), b: (i32, i32), c: (i32, i32)) -> bool {
    let pa = edge(a, b, p);
    let pb = edge(b, c, p);
    let pc = edge(c, a, p);

    let has_neg = pa < 0 || pb < 0 || pc < 0;
    let has_pos = pa > 0 || pb > 0 || pc > 0;
    !(has_neg && has_pos)
}

fn edge(a: (i32, i32), b: (i32, i32), p: (i32, i32)) -> i32 {
    (p.0 - a.0) * (b.1 - a.1) - (p.1 - a.1) * (b.0 - a.0)
}
