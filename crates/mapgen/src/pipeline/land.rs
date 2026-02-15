use std::collections::VecDeque;

use rand_chacha::{
    ChaCha12Rng,
    rand_core::{Rng, SeedableRng},
};

use crate::pipeline::{
    helpers::{
        ConstraintsConfig, DraftConfig, LandGlobalConfig, RepairConfig, landmasses_config,
        neighbors_odd_r,
    },
    map_sizes::MapSizes,
    map_types::MapTypes,
};

#[derive(Default)]
/// Measurements collected from a generated landmask to decide whether repairs are needed.
struct LandAnalysis {
    land_ratio: f32,
    largest_ratio: f32,
    second_ratio: f32,
    n_components: usize,
    n_islands: usize,
    n_lakes: usize,
    land_tiles: usize,
    largest_component_idx: Option<usize>,
    land_component_sizes: Vec<usize>,
    land_component_ids: Vec<usize>,
    ocean_mask: Vec<bool>,
}

#[derive(Clone, Copy)]
/// Internal enum describing the repair behavior for each map style.
enum RepairStyle {
    Continents,
    SmallContinents,
    IslandContinents,
    Pangea,
    Terra,
    Mirror,
}

/// Generate land for the requested map type.
pub fn generate_landmasses(seed: u64, size: &MapSizes, map_type: MapTypes) -> Vec<u8> {
    match map_type {
        MapTypes::Continents => generate_continents(seed, size),
        MapTypes::SmallContinents => generate_small_continents(seed, size),
        MapTypes::IslandsContinents => generate_island_continents(seed, size),
        MapTypes::Pangea => generate_pangea(seed, size),
        MapTypes::Mirror => generate_mirror(seed, size),
        MapTypes::Terra => generate_terra(seed, size),
    }
}

/// Generate a continents-style map with deterministic analyze/repair.
pub fn generate_continents(seed: u64, size: &MapSizes) -> Vec<u8> {
    let cfg = landmasses_config();
    let mut rng = ChaCha12Rng::seed_from_u64(seed);

    let mut grid = generate_zoom_draft(
        &mut child_rng(&mut rng),
        size,
        &cfg.global,
        &cfg.continents.draft,
        None,
    );

    run_repair_loop(
        &mut rng,
        size,
        &cfg.global,
        &cfg.continents.constraints,
        &cfg.continents.repair,
        RepairStyle::Continents,
        &mut grid,
    );
    grid
}

/// Generate a small-continents map with deterministic analyze/repair.
pub fn generate_small_continents(seed: u64, size: &MapSizes) -> Vec<u8> {
    let cfg = landmasses_config();
    let mut rng = ChaCha12Rng::seed_from_u64(seed);

    let mut grid = generate_zoom_draft(
        &mut child_rng(&mut rng),
        size,
        &cfg.global,
        &cfg.small_continents.draft,
        None,
    );

    run_repair_loop(
        &mut rng,
        size,
        &cfg.global,
        &cfg.small_continents.constraints,
        &cfg.small_continents.repair,
        RepairStyle::SmallContinents,
        &mut grid,
    );
    grid
}

/// Generate an island-continents (archipelago-like) map with deterministic analyze/repair.
pub fn generate_island_continents(seed: u64, size: &MapSizes) -> Vec<u8> {
    let cfg = landmasses_config();
    let mut rng = ChaCha12Rng::seed_from_u64(seed);

    let mut grid = generate_zoom_draft(
        &mut child_rng(&mut rng),
        size,
        &cfg.global,
        &cfg.island_continents.draft,
        None,
    );

    run_repair_loop(
        &mut rng,
        size,
        &cfg.global,
        &cfg.island_continents.constraints,
        &cfg.island_continents.repair,
        RepairStyle::IslandContinents,
        &mut grid,
    );
    grid
}

/// Generate a pangea-style map with deterministic analyze/repair.
pub fn generate_pangea(seed: u64, size: &MapSizes) -> Vec<u8> {
    let cfg = landmasses_config();
    let mut rng = ChaCha12Rng::seed_from_u64(seed);

    let mut grid = generate_zoom_draft(
        &mut child_rng(&mut rng),
        size,
        &cfg.global,
        &cfg.pangea.draft,
        None,
    );

    run_repair_loop(
        &mut rng,
        size,
        &cfg.global,
        &cfg.pangea.constraints,
        &cfg.pangea.repair,
        RepairStyle::Pangea,
        &mut grid,
    );
    grid
}

/// Generate a terra map with old/new world split by a deterministic ocean barrier.
pub fn generate_terra(seed: u64, size: &MapSizes) -> Vec<u8> {
    let cfg = landmasses_config();
    let mut rng = ChaCha12Rng::seed_from_u64(seed);
    let (width, height) = size.dimensions();

    let vertical = (rng.next_u32() & 1) == 0;
    let barrier_span = cfg.terra.barrier_max.saturating_sub(cfg.terra.barrier_min) + 1;
    let mut barrier_w = cfg.terra.barrier_min + (rng.next_u32() as usize % barrier_span.max(1));
    barrier_w = barrier_w.min(width.saturating_sub(2).max(1));

    let mut old_side = vec![true; width * height];
    let mut new_side = vec![true; width * height];

    if vertical {
        let split = width / 2;
        let start = split.saturating_sub(barrier_w / 2);
        let end = (start + barrier_w).min(width);
        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;
                if x >= end {
                    old_side[idx] = false;
                }
                if x < start {
                    new_side[idx] = false;
                }
                if (start..end).contains(&x) {
                    old_side[idx] = false;
                    new_side[idx] = false;
                }
            }
        }
    } else {
        let split = height / 2;
        let start = split.saturating_sub(barrier_w / 2);
        let end = (start + barrier_w).min(height);
        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;
                if y >= end {
                    old_side[idx] = false;
                }
                if y < start {
                    new_side[idx] = false;
                }
                if (start..end).contains(&y) {
                    old_side[idx] = false;
                    new_side[idx] = false;
                }
            }
        }
    }

    let old_world = generate_zoom_draft(
        &mut child_rng(&mut rng),
        size,
        &cfg.global,
        &cfg.terra.old_world.draft,
        Some(&old_side),
    );
    let new_world = generate_zoom_draft(
        &mut child_rng(&mut rng),
        size,
        &cfg.global,
        &cfg.terra.new_world.draft,
        Some(&new_side),
    );

    let mut grid = vec![0u8; width * height];
    for i in 0..grid.len() {
        if old_world[i] == 1 || new_world[i] == 1 {
            grid[i] = 1;
        }
    }

    enforce_border_water(&mut grid, width, height);

    run_repair_loop(
        &mut rng,
        size,
        &cfg.global,
        &cfg.terra.merged_constraints,
        &cfg.terra.merged_repair,
        RepairStyle::Terra,
        &mut grid,
    );

    // Keep terra to exactly two major worlds split by ocean.
    enforce_terra_two_worlds(
        &mut grid,
        width,
        height,
        vertical,
        barrier_w.max(3),
        &old_side,
        &new_side,
        &cfg.global,
        &mut rng,
    );

    if vertical {
        let split = width / 2;
        let effective_w = barrier_w.max(3);
        let start = split.saturating_sub(effective_w / 2);
        let end = (start + effective_w).min(width);
        for y in 0..height {
            for x in start..end {
                grid[y * width + x] = 0;
            }
        }
    } else {
        let split = height / 2;
        let effective_w = barrier_w.max(3);
        let start = split.saturating_sub(effective_w / 2);
        let end = (start + effective_w).min(height);
        for y in start..end {
            for x in 0..width {
                grid[y * width + x] = 0;
            }
        }
    }

    enforce_border_water(&mut grid, width, height);
    grid
}

/// Generate a perfectly mirrored map by creating and repairing half, then reflecting.
pub fn generate_mirror(seed: u64, size: &MapSizes) -> Vec<u8> {
    let cfg = landmasses_config();
    let mut rng = ChaCha12Rng::seed_from_u64(seed);
    let (width, height) = size.dimensions();
    let half_w = width.div_ceil(2);

    let full = generate_zoom_draft(
        &mut child_rng(&mut rng),
        size,
        &cfg.global,
        &cfg.mirror.base.draft,
        None,
    );

    let mut half = vec![0u8; half_w * height];
    for y in 0..height {
        for x in 0..half_w {
            half[y * half_w + x] = full[y * width + x];
        }
    }

    for _ in 0..cfg.mirror.half_smoothing_passes {
        let mut next = half.clone();
        for y in 0..height {
            for x in 0..half_w {
                let idx = y * half_w + x;
                let mut land_n = 0;
                let mut water_n = 0;
                for (nx, ny) in neighbors_odd_r(x, y, half_w, height) {
                    if half[ny * half_w + nx] == 1 {
                        land_n += 1;
                    } else {
                        water_n += 1;
                    }
                }
                if land_n >= 4 {
                    next[idx] = 1;
                } else if water_n >= 4 {
                    next[idx] = 0;
                }
            }
        }
        half = next;
    }

    let mut grid = vec![0u8; width * height];
    mirror_vertical_into(&half, &mut grid, width, height);

    run_repair_loop(
        &mut rng,
        size,
        &cfg.global,
        &cfg.mirror.base.constraints,
        &cfg.mirror.base.repair,
        RepairStyle::Mirror,
        &mut grid,
    );

    enforce_vertical_mirror(&mut grid, width, height);
    enforce_border_water(&mut grid, width, height);
    grid
}

/// Create a deterministic child RNG from the parent RNG stream.
fn child_rng(parent: &mut ChaCha12Rng) -> ChaCha12Rng {
    ChaCha12Rng::seed_from_u64(parent.next_u64())
}

/// Analyze current map, apply style-specific repairs, and stop after acceptance or max iterations.
fn run_repair_loop(
    rng: &mut ChaCha12Rng,
    size: &MapSizes,
    global: &LandGlobalConfig,
    constraints: &ConstraintsConfig,
    repair: &RepairConfig,
    style: RepairStyle,
    grid: &mut Vec<u8>,
) {
    let (width, height) = size.dimensions();
    let island_max = dynamic_island_max(size, global);
    let mid_max = dynamic_mid_max(size, global);

    for _ in 0..global.max_repair_iters {
        let analysis = analyze_landmask(grid, width, height, island_max, mid_max, global.min_lake_size);
        if satisfies(&analysis, constraints) {
            break;
        }

        match style {
            RepairStyle::Continents => {
                if analysis.largest_ratio > repair.largest_carve_trigger_ratio {
                    let over = (analysis.largest_ratio - repair.largest_carve_target_ratio).max(0.0);
                    let map_scale = (width * height) as f32 / (84.0 * 54.0);
                    let k = ((repair.largest_carve_base_count as f32
                        + (over * repair.largest_carve_scale))
                        * map_scale.max(1.0))
                        .ceil() as usize;
                    carve_straits(grid, width, height, &analysis, rng, k);
                }
                if analysis.n_components < constraints.min_components {
                    let missing = constraints.min_components - analysis.n_components;
                    let map_scale = (width * height) as f32 / (84.0 * 54.0);
                    let base = ((repair.channel_carve_count.max(4) as f32) * map_scale.max(1.0))
                        .ceil() as usize;
                    channel_carve(grid, width, height, &analysis, rng, base * missing);
                }
                if analysis.n_islands < constraints.min_islands {
                    let missing = constraints.min_islands - analysis.n_islands;
                    sprinkle_islands(
                        grid,
                        width,
                        height,
                        rng,
                        missing,
                        repair.island_min_blob,
                        repair.island_max_blob,
                    );
                }
                if analysis.n_lakes < constraints.min_lakes {
                    carve_lakes(
                        grid,
                        width,
                        height,
                        &analysis,
                        rng,
                        constraints.min_lakes - analysis.n_lakes,
                        repair.lake_blob_min,
                        repair.lake_blob_max,
                    );
                }
            }
            RepairStyle::SmallContinents => {
                if analysis.largest_ratio > repair.largest_carve_trigger_ratio {
                    let over = (analysis.largest_ratio - repair.largest_carve_target_ratio).max(0.0);
                    let k = repair.largest_carve_base_count + (over * repair.largest_carve_scale).ceil() as usize;
                    carve_straits(grid, width, height, &analysis, rng, k);
                }
                if analysis.n_components < constraints.min_components && repair.channel_carve_count > 0 {
                    channel_carve(grid, width, height, &analysis, rng, repair.channel_carve_count);
                }
                if analysis.n_islands < constraints.min_islands {
                    let missing = constraints.min_islands - analysis.n_islands;
                    sprinkle_islands(
                        grid,
                        width,
                        height,
                        rng,
                        missing,
                        repair.island_min_blob,
                        repair.island_max_blob,
                    );
                }
                if analysis.n_lakes < constraints.min_lakes {
                    carve_lakes(
                        grid,
                        width,
                        height,
                        &analysis,
                        rng,
                        constraints.min_lakes - analysis.n_lakes,
                        repair.lake_blob_min,
                        repair.lake_blob_max,
                    );
                }
            }
            RepairStyle::IslandContinents => {
                let cap = (analysis.land_tiles as f32 * repair.erode_cap_ratio) as usize;
                erode_largest_component(grid, width, height, &analysis, rng, cap);

                if analysis.n_islands < constraints.min_islands {
                    let missing = constraints.min_islands - analysis.n_islands;
                    let count = missing.max(repair.island_extra_missing_floor);
                    sprinkle_islands(
                        grid,
                        width,
                        height,
                        rng,
                        count,
                        repair.island_min_blob,
                        repair.island_max_blob,
                    );
                }
            }
            RepairStyle::Pangea => {
                if analysis.largest_ratio < constraints.min_largest_ratio {
                    fill_internal_straits(
                        grid,
                        width,
                        height,
                        &analysis,
                        rng,
                        repair.pangea_fill_internal_count,
                    );
                    connect_to_largest(
                        grid,
                        width,
                        height,
                        &analysis,
                        rng,
                        repair.pangea_connect_count,
                    );
                }
                if analysis.n_components > constraints.max_components {
                    connect_to_largest(
                        grid,
                        width,
                        height,
                        &analysis,
                        rng,
                        repair.pangea_connect_when_split,
                    );
                }
                if analysis.n_islands < constraints.min_islands {
                    sprinkle_islands(
                        grid,
                        width,
                        height,
                        rng,
                        1,
                        repair.island_min_blob,
                        repair.island_max_blob,
                    );
                }
                if analysis.n_lakes < constraints.min_lakes {
                    carve_lakes(
                        grid,
                        width,
                        height,
                        &analysis,
                        rng,
                        constraints.min_lakes - analysis.n_lakes,
                        repair.lake_blob_min,
                        repair.lake_blob_max,
                    );
                }
            }
            RepairStyle::Terra => {
                if analysis.second_ratio < 0.20 {
                    grow_land(grid, width, height, rng, repair.terra_grow_budget);
                }
                if analysis.n_islands < constraints.min_islands {
                    sprinkle_islands(
                        grid,
                        width,
                        height,
                        rng,
                        repair.island_extra_missing_floor,
                        repair.island_min_blob,
                        repair.island_max_blob,
                    );
                }
                if analysis.n_lakes < constraints.min_lakes {
                    carve_lakes(
                        grid,
                        width,
                        height,
                        &analysis,
                        rng,
                        constraints.min_lakes - analysis.n_lakes,
                        repair.lake_blob_min,
                        repair.lake_blob_max,
                    );
                }
            }
            RepairStyle::Mirror => {
                if analysis.largest_ratio > repair.largest_carve_trigger_ratio {
                    carve_straits(grid, width, height, &analysis, rng, repair.largest_carve_base_count);
                }
                if analysis.n_islands < constraints.min_islands {
                    sprinkle_islands(
                        grid,
                        width,
                        height,
                        rng,
                        repair.island_extra_missing_floor,
                        repair.island_min_blob,
                        repair.island_max_blob,
                    );
                }
                enforce_vertical_mirror(grid, width, height);
            }
        }

        adjust_land_ratio(
            grid,
            width,
            height,
            rng,
            constraints.min_land_ratio,
            constraints.max_land_ratio,
            repair.land_ratio_adjust_cap_divisor,
        );
        enforce_border_water(grid, width, height);

        if let RepairStyle::Mirror = style {
            enforce_vertical_mirror(grid, width, height);
        }
    }

    // Final hard constraint pass: enforce configured land-ratio bounds directly.
    force_land_ratio(
        grid,
        width,
        height,
        rng,
        constraints.min_land_ratio,
        constraints.max_land_ratio,
    );
    enforce_border_water(grid, width, height);

    // Final hard topology pass: enforce minimum land component count for map styles that need it.
    if constraints.min_components > 1 {
        ensure_min_components(grid, width, height, global, constraints, rng);
    }

    // Component splitting can slightly move land ratio, so enforce ratio one more time.
    force_land_ratio(
        grid,
        width,
        height,
        rng,
        constraints.min_land_ratio,
        constraints.max_land_ratio,
    );
    enforce_border_water(grid, width, height);

    if let RepairStyle::Mirror = style {
        enforce_vertical_mirror(grid, width, height);
        enforce_border_water(grid, width, height);
    }
}

/// Build an initial land draft using coarse seeding, zoom, and smoothing.
fn generate_zoom_draft(
    rng: &mut ChaCha12Rng,
    size: &MapSizes,
    global: &LandGlobalConfig,
    params: &DraftConfig,
    area_mask: Option<&[bool]>,
) -> Vec<u8> {
    let (width, height) = size.dimensions();

    let mut w = width.div_ceil(global.base_factor).max(2);
    let mut h = height.div_ceil(global.base_factor).max(2);
    let mut grid = vec![0u8; w * h];

    let center_x = (w as f32 - 1.0) * (0.35 + 0.3 * (rng.next_u32() as f32 / u32::MAX as f32));
    let center_y = (h as f32 - 1.0) * (0.35 + 0.3 * (rng.next_u32() as f32 / u32::MAX as f32));

    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            let border = x == 0 || x + 1 == w || y == 0 || y + 1 == h;
            if border {
                grid[idx] = 0;
                continue;
            }

            let mut p = params.base_land_percent as f32;
            if params.center_bias > 0.0 {
                let dx = (x as f32 - center_x) / (w as f32 * 0.45);
                let dy = (y as f32 - center_y) / (h as f32 * 0.45);
                let d2 = dx * dx + dy * dy;
                let boost = (1.0 - d2).max(0.0) * 40.0 * params.center_bias;
                p += boost;
            }

            grid[idx] = if ((rng.next_u32() % 100) as f32) < p {
                1
            } else {
                0
            };
        }
    }

    while w < width || h < height {
        let new_w = (w * 2).min(width);
        let new_h = (h * 2).min(height);
        let mut next = vec![0u8; new_w * new_h];

        for ny in 0..new_h {
            for nx in 0..new_w {
                let px = (nx / 2).min(w - 1);
                let py = (ny / 2).min(h - 1);
                let pe = (px + 1).min(w - 1);
                let ps = (py + 1).min(h - 1);

                let parent = grid[py * w + px];
                let east = grid[py * w + pe];
                let south = grid[ps * w + px];
                let diag = grid[ps * w + pe];

                let land_votes = parent + east + south + diag;
                let mut value = if land_votes > 2 {
                    1
                } else if land_votes < 2 {
                    0
                } else {
                    parent
                };

                let mixed = land_votes > 0 && land_votes < 4;
                if mixed {
                    if rng.next_u32() % 100 < params.fuzzy_flip_percent {
                        value = 1 - value;
                    }
                    if value == 0 && rng.next_u32() % 100 < params.coast_island_percent {
                        value = 1;
                    }
                }
                next[ny * new_w + nx] = value;
            }
        }

        enforce_border_water(&mut next, new_w, new_h);
        grid = next;
        w = new_w;
        h = new_h;
    }

    for _ in 0..params.smoothing_passes {
        let mut next = grid.clone();
        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;
                let mut land_n = 0;
                let mut water_n = 0;
                for (nx, ny) in neighbors_odd_r(x, y, width, height) {
                    if grid[ny * width + nx] == 1 {
                        land_n += 1;
                    } else {
                        water_n += 1;
                    }
                }
                if land_n >= 4 {
                    next[idx] = 1;
                } else if water_n >= 4 {
                    next[idx] = 0;
                }
            }
        }
        enforce_border_water(&mut next, width, height);
        grid = next;
    }

    if let Some(mask) = area_mask {
        for i in 0..grid.len() {
            if !mask[i] {
                grid[i] = 0;
            }
        }
    }

    enforce_border_water(&mut grid, width, height);
    grid
}

/// Analyze a landmask and return all stats needed by the repair loop.
fn analyze_landmask(
    grid: &[u8],
    width: usize,
    height: usize,
    island_max: usize,
    mid_max: usize,
    min_lake_size: usize,
) -> LandAnalysis {
    let n = width * height;
    let mut land_component_ids = vec![usize::MAX; n];
    let mut land_component_sizes = Vec::new();
    let mut q = VecDeque::new();

    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            if grid[idx] != 1 || land_component_ids[idx] != usize::MAX {
                continue;
            }

            let comp_id = land_component_sizes.len();
            let mut size = 0usize;
            land_component_ids[idx] = comp_id;
            q.push_back((x, y));

            while let Some((cx, cy)) = q.pop_front() {
                size += 1;
                for (nx, ny) in neighbors_odd_r(cx, cy, width, height) {
                    let nidx = ny * width + nx;
                    if grid[nidx] == 1 && land_component_ids[nidx] == usize::MAX {
                        land_component_ids[nidx] = comp_id;
                        q.push_back((nx, ny));
                    }
                }
            }

            land_component_sizes.push(size);
        }
    }

    let land_tiles = land_component_sizes.iter().sum::<usize>();
    let land_ratio = if n == 0 { 0.0 } else { land_tiles as f32 / n as f32 };

    let mut largest_component_idx = None;
    let mut largest = 0usize;
    let mut second = 0usize;
    for (i, &sz) in land_component_sizes.iter().enumerate() {
        if sz > largest {
            second = largest;
            largest = sz;
            largest_component_idx = Some(i);
        } else if sz > second {
            second = sz;
        }
    }

    let largest_ratio = if land_tiles > 0 { largest as f32 / land_tiles as f32 } else { 0.0 };
    let second_ratio = if land_tiles > 0 { second as f32 / land_tiles as f32 } else { 0.0 };

    let n_islands = land_component_sizes
        .iter()
        .filter(|&&s| s <= island_max || (s <= mid_max && s < island_max * 2))
        .count();

    let (ocean_mask, n_lakes) = analyze_water(grid, width, height, min_lake_size);

    LandAnalysis {
        land_ratio,
        largest_ratio,
        second_ratio,
        n_components: land_component_sizes.len(),
        n_islands,
        n_lakes,
        land_tiles,
        largest_component_idx,
        land_component_sizes,
        land_component_ids,
        ocean_mask,
    }
}

/// Analyze water components, classify ocean, and count lakes.
fn analyze_water(grid: &[u8], width: usize, height: usize, min_lake_size: usize) -> (Vec<bool>, usize) {
    let n = width * height;
    let mut water_component_ids = vec![usize::MAX; n];
    let mut water_component_sizes = Vec::new();
    let mut touches_border = Vec::new();

    let mut q = VecDeque::new();
    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            if grid[idx] != 0 || water_component_ids[idx] != usize::MAX {
                continue;
            }

            let comp_id = water_component_sizes.len();
            let mut size = 0usize;
            let mut border = false;

            water_component_ids[idx] = comp_id;
            q.push_back((x, y));

            while let Some((cx, cy)) = q.pop_front() {
                size += 1;
                if cx == 0 || cy == 0 || cx + 1 == width || cy + 1 == height {
                    border = true;
                }

                for (nx, ny) in neighbors_odd_r(cx, cy, width, height) {
                    let nidx = ny * width + nx;
                    if grid[nidx] == 0 && water_component_ids[nidx] == usize::MAX {
                        water_component_ids[nidx] = comp_id;
                        q.push_back((nx, ny));
                    }
                }
            }

            water_component_sizes.push(size);
            touches_border.push(border);
        }
    }

    let mut ocean_mask = vec![false; n];
    for i in 0..n {
        if grid[i] != 0 {
            continue;
        }
        let comp = water_component_ids[i];
        if touches_border[comp] {
            ocean_mask[i] = true;
        }
    }

    let n_lakes = water_component_sizes
        .iter()
        .enumerate()
        .filter(|(i, sz)| !touches_border[*i] && **sz >= min_lake_size)
        .count();

    (ocean_mask, n_lakes)
}

/// Check whether the current map satisfies all configured constraints.
fn satisfies(a: &LandAnalysis, c: &ConstraintsConfig) -> bool {
    a.land_ratio >= c.min_land_ratio
        && a.land_ratio <= c.max_land_ratio
        && a.largest_ratio >= c.min_largest_ratio
        && a.largest_ratio <= c.max_largest_ratio
        && a.n_components >= c.min_components
        && a.n_components <= c.max_components
        && a.n_islands >= c.min_islands
        && a.n_lakes >= c.min_lakes
        && a.n_lakes <= c.max_lakes
}

/// Carve coastal choke points on the largest component to split oversized landmasses.
fn carve_straits(
    grid: &mut [u8],
    width: usize,
    height: usize,
    analysis: &LandAnalysis,
    rng: &mut ChaCha12Rng,
    k: usize,
) {
    let Some(largest_id) = analysis.largest_component_idx else {
        return;
    };

    let mut candidates: Vec<(i32, u64, usize)> = Vec::new();
    for y in 1..height.saturating_sub(1) {
        for x in 1..width.saturating_sub(1) {
            let idx = y * width + x;
            if grid[idx] != 1 || analysis.land_component_ids[idx] != largest_id {
                continue;
            }

            let mut land_n = 0i32;
            let mut water_n = 0i32;
            for (nx, ny) in neighbors_odd_r(x, y, width, height) {
                if grid[ny * width + nx] == 1 {
                    land_n += 1;
                } else {
                    water_n += 1;
                }
            }

            if water_n >= 2 && land_n >= 2 {
                let score = water_n * 10 + land_n;
                candidates.push((score, rng.next_u64(), idx));
            }
        }
    }

    candidates.sort_unstable_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    for (_, _, idx) in candidates.into_iter().take(k) {
        grid[idx] = 0;
    }
}

/// Carve channels in mixed coastline regions to increase component count.
fn channel_carve(
    grid: &mut [u8],
    width: usize,
    height: usize,
    analysis: &LandAnalysis,
    rng: &mut ChaCha12Rng,
    k: usize,
) {
    let mut candidates: Vec<(i32, u64, usize)> = Vec::new();

    for y in 1..height.saturating_sub(1) {
        for x in 1..width.saturating_sub(1) {
            let idx = y * width + x;
            if grid[idx] != 1 {
                continue;
            }

            let mut land_n = 0i32;
            let mut water_n = 0i32;
            for (nx, ny) in neighbors_odd_r(x, y, width, height) {
                if grid[ny * width + nx] == 1 {
                    land_n += 1;
                } else {
                    water_n += 1;
                }
            }

            if water_n >= 2 && land_n >= 2 {
                let comp_bias = (analysis.land_component_ids[idx] % 7) as i32;
                let score = water_n * 12 + comp_bias;
                candidates.push((score, rng.next_u64(), idx));
            }
        }
    }

    candidates.sort_unstable_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    for (_, _, idx) in candidates.into_iter().take(k) {
        grid[idx] = 0;
    }
}

/// Add new island blobs in ocean tiles far from existing land.
fn sprinkle_islands(
    grid: &mut [u8],
    width: usize,
    height: usize,
    rng: &mut ChaCha12Rng,
    count: usize,
    min_blob: usize,
    max_blob: usize,
) {
    let mut candidates: Vec<(u64, usize)> = Vec::new();

    for y in 1..height.saturating_sub(1) {
        for x in 1..width.saturating_sub(1) {
            let idx = y * width + x;
            if grid[idx] != 0 {
                continue;
            }
            if neighbors_odd_r(x, y, width, height)
                .into_iter()
                .any(|(nx, ny)| grid[ny * width + nx] == 1)
            {
                continue;
            }
            candidates.push((rng.next_u64(), idx));
        }
    }

    candidates.sort_unstable_by_key(|v| v.0);
    let blob_span = max_blob.saturating_sub(min_blob) + 1;

    let mut placed = 0usize;
    for (_, center_idx) in candidates {
        if placed >= count || grid[center_idx] != 0 {
            continue;
        }
        let blob_size = min_blob + (rng.next_u32() as usize % blob_span.max(1));
        grow_blob_from_center(grid, width, height, center_idx, 1, blob_size, rng);
        placed += 1;
    }
}

/// Carve inland lake blobs away from ocean-connected water.
fn carve_lakes(
    grid: &mut [u8],
    width: usize,
    height: usize,
    analysis: &LandAnalysis,
    rng: &mut ChaCha12Rng,
    count: usize,
    min_blob: usize,
    max_blob: usize,
) {
    let dist = inland_distance_to_ocean(grid, &analysis.ocean_mask, width, height);
    let mut candidates: Vec<(u16, u64, usize)> = Vec::new();

    for y in 1..height.saturating_sub(1) {
        for x in 1..width.saturating_sub(1) {
            let idx = y * width + x;
            if grid[idx] == 1 && dist[idx] >= 3 {
                candidates.push((dist[idx], rng.next_u64(), idx));
            }
        }
    }

    candidates.sort_unstable_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    let blob_span = max_blob.saturating_sub(min_blob) + 1;

    for (_, _, idx) in candidates.into_iter().take(count) {
        let blob_size = min_blob + (rng.next_u32() as usize % blob_span.max(1));
        grow_blob_from_center(grid, width, height, idx, 0, blob_size, rng);
    }
}

/// Erode exposed coastal tiles from the largest component until it is under a target cap.
fn erode_largest_component(
    grid: &mut [u8],
    width: usize,
    height: usize,
    analysis: &LandAnalysis,
    rng: &mut ChaCha12Rng,
    cap: usize,
) {
    let Some(largest_id) = analysis.largest_component_idx else {
        return;
    };

    let largest_size = analysis.land_component_sizes[largest_id];
    if largest_size <= cap {
        return;
    }

    let mut candidates: Vec<(i32, u64, usize)> = Vec::new();

    for y in 1..height.saturating_sub(1) {
        for x in 1..width.saturating_sub(1) {
            let idx = y * width + x;
            if grid[idx] != 1 || analysis.land_component_ids[idx] != largest_id {
                continue;
            }

            let water_n = neighbors_odd_r(x, y, width, height)
                .into_iter()
                .filter(|(nx, ny)| grid[ny * width + nx] == 0)
                .count() as i32;

            if water_n >= 3 {
                candidates.push((water_n, rng.next_u64(), idx));
            }
        }
    }

    candidates.sort_unstable_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    let remove_count = (largest_size - cap).min(candidates.len());
    for (_, _, idx) in candidates.into_iter().take(remove_count) {
        grid[idx] = 0;
    }
}

/// Fill narrow channels inside the main continent to strengthen a pangea shape.
fn fill_internal_straits(
    grid: &mut [u8],
    width: usize,
    height: usize,
    analysis: &LandAnalysis,
    rng: &mut ChaCha12Rng,
    max_fill: usize,
) {
    let Some(largest_id) = analysis.largest_component_idx else {
        return;
    };

    let mut candidates: Vec<(i32, u64, usize)> = Vec::new();
    for y in 1..height.saturating_sub(1) {
        for x in 1..width.saturating_sub(1) {
            let idx = y * width + x;
            if grid[idx] != 0 {
                continue;
            }

            let mut land_n = 0i32;
            let mut near_largest = 0i32;
            for (nx, ny) in neighbors_odd_r(x, y, width, height) {
                let nidx = ny * width + nx;
                if grid[nidx] == 1 {
                    land_n += 1;
                    if analysis.land_component_ids[nidx] == largest_id {
                        near_largest += 1;
                    }
                }
            }

            if land_n >= 4 && near_largest >= 2 {
                let score = land_n * 10 + near_largest;
                candidates.push((score, rng.next_u64(), idx));
            }
        }
    }

    candidates.sort_unstable_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    for (_, _, idx) in candidates.into_iter().take(max_fill) {
        grid[idx] = 1;
    }
}

/// Connect smaller land components to the largest one using soft land bridges.
fn connect_to_largest(
    grid: &mut [u8],
    width: usize,
    height: usize,
    analysis: &LandAnalysis,
    rng: &mut ChaCha12Rng,
    max_connections: usize,
) {
    let Some(largest_id) = analysis.largest_component_idx else {
        return;
    };

    let largest_center = component_center(analysis, largest_id, width);
    let mut others: Vec<(usize, u64, usize)> = Vec::new();

    for comp in 0..analysis.land_component_sizes.len() {
        if comp == largest_id {
            continue;
        }
        let center = component_center(analysis, comp, width);
        let dist = hex_distance_offset(largest_center, center);
        others.push((dist, rng.next_u64(), comp));
    }

    others.sort_unstable_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    for (_, _, comp) in others.into_iter().take(max_connections) {
        let center = component_center(analysis, comp, width);
        draw_soft_line(grid, width, height, largest_center, center, 1);
    }
}

/// Expand land from near-coast candidates to increase total land ratio.
fn grow_land(grid: &mut [u8], width: usize, height: usize, rng: &mut ChaCha12Rng, budget: usize) {
    let mut candidates: Vec<(i32, u64, usize)> = Vec::new();

    for y in 1..height.saturating_sub(1) {
        for x in 1..width.saturating_sub(1) {
            let idx = y * width + x;
            if grid[idx] != 0 {
                continue;
            }

            let land_n = neighbors_odd_r(x, y, width, height)
                .into_iter()
                .filter(|(nx, ny)| grid[ny * width + nx] == 1)
                .count() as i32;

            if land_n >= 2 {
                candidates.push((land_n, rng.next_u64(), idx));
            }
        }
    }

    candidates.sort_unstable_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    for (_, _, idx) in candidates.into_iter().take(budget) {
        grid[idx] = 1;
    }
}

/// Adjust global land ratio by growing/shrinking near-coast tiles.
fn adjust_land_ratio(
    grid: &mut [u8],
    width: usize,
    height: usize,
    rng: &mut ChaCha12Rng,
    min_ratio: f32,
    max_ratio: f32,
    cap_divisor: usize,
) {
    let total = width * height;
    let land = grid.iter().filter(|&&v| v == 1).count();
    let ratio = land as f32 / total as f32;
    let cap = (total / cap_divisor.max(1)).max(1);

    if ratio < min_ratio {
        let target = (min_ratio * total as f32).ceil() as usize;
        let need = target.saturating_sub(land).min(cap);
        grow_land(grid, width, height, rng, need);
    } else if ratio > max_ratio {
        let target = (max_ratio * total as f32).floor() as usize;
        let need = land.saturating_sub(target).min(cap);

        let mut candidates: Vec<(i32, u64, usize)> = Vec::new();
        for y in 1..height.saturating_sub(1) {
            for x in 1..width.saturating_sub(1) {
                let idx = y * width + x;
                if grid[idx] != 1 {
                    continue;
                }

                let water_n = neighbors_odd_r(x, y, width, height)
                    .into_iter()
                    .filter(|(nx, ny)| grid[ny * width + nx] == 0)
                    .count() as i32;

                if water_n >= 2 {
                    candidates.push((water_n, rng.next_u64(), idx));
                }
            }
        }

        candidates.sort_unstable_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
        for (_, _, idx) in candidates.into_iter().take(need) {
            grid[idx] = 0;
        }
    }
}

/// Force the final land ratio into [min_ratio, max_ratio] by directly flipping coastal-adjacent tiles.
fn force_land_ratio(
    grid: &mut [u8],
    width: usize,
    height: usize,
    rng: &mut ChaCha12Rng,
    min_ratio: f32,
    max_ratio: f32,
) {
    let total = width * height;
    let min_land = (min_ratio * total as f32).ceil() as usize;
    let max_land = (max_ratio * total as f32).floor() as usize;

    // Grow phase.
    loop {
        let land = grid.iter().filter(|&&v| v == 1).count();
        if land >= min_land {
            break;
        }

        let mut coastal_water: Vec<(u64, usize)> = Vec::new();
        let mut any_water: Vec<(u64, usize)> = Vec::new();
        for y in 1..height.saturating_sub(1) {
            for x in 1..width.saturating_sub(1) {
                let idx = y * width + x;
                if grid[idx] != 0 {
                    continue;
                }

                any_water.push((rng.next_u64(), idx));
                let near_land = neighbors_odd_r(x, y, width, height)
                    .into_iter()
                    .any(|(nx, ny)| grid[ny * width + nx] == 1);
                if near_land {
                    coastal_water.push((rng.next_u64(), idx));
                }
            }
        }

        if coastal_water.is_empty() && any_water.is_empty() {
            break;
        }

        if !coastal_water.is_empty() {
            coastal_water.sort_unstable_by_key(|v| v.0);
            grid[coastal_water[0].1] = 1;
        } else {
            any_water.sort_unstable_by_key(|v| v.0);
            grid[any_water[0].1] = 1;
        }
    }

    // Shrink phase.
    loop {
        let land = grid.iter().filter(|&&v| v == 1).count();
        if land <= max_land {
            break;
        }

        let mut coastal_land: Vec<(u64, usize)> = Vec::new();
        let mut any_land: Vec<(u64, usize)> = Vec::new();
        for y in 1..height.saturating_sub(1) {
            for x in 1..width.saturating_sub(1) {
                let idx = y * width + x;
                if grid[idx] != 1 {
                    continue;
                }

                any_land.push((rng.next_u64(), idx));
                let near_water = neighbors_odd_r(x, y, width, height)
                    .into_iter()
                    .any(|(nx, ny)| grid[ny * width + nx] == 0);
                if near_water {
                    coastal_land.push((rng.next_u64(), idx));
                }
            }
        }

        if coastal_land.is_empty() && any_land.is_empty() {
            break;
        }

        if !coastal_land.is_empty() {
            coastal_land.sort_unstable_by_key(|v| v.0);
            grid[coastal_land[0].1] = 0;
        } else {
            any_land.sort_unstable_by_key(|v| v.0);
            grid[any_land[0].1] = 0;
        }
    }
}

/// Ensure maps that require multiple land components do not end as a single supercontinent.
fn ensure_min_components(
    grid: &mut [u8],
    width: usize,
    height: usize,
    global: &LandGlobalConfig,
    constraints: &ConstraintsConfig,
    rng: &mut ChaCha12Rng,
) {
    let island_max = dynamic_island_max(&size_from_dims(width, height), global);
    let mid_max = dynamic_mid_max(&size_from_dims(width, height), global);

    for _ in 0..8 {
        let analysis = analyze_landmask(grid, width, height, island_max, mid_max, global.min_lake_size);
        if analysis.n_components >= constraints.min_components {
            break;
        }

        let missing = constraints.min_components - analysis.n_components;
        let map_scale = (width * height) as f32 / (84.0 * 54.0);
        let k = ((8.0 * map_scale.max(1.0)).ceil() as usize) * missing;

        carve_straits(grid, width, height, &analysis, rng, k);
        channel_carve(grid, width, height, &analysis, rng, (k / 2).max(1));
        enforce_border_water(grid, width, height);
    }
}

/// Enforce terra outcomes: exactly two components, one on each side of the split.
fn enforce_terra_two_worlds(
    grid: &mut [u8],
    width: usize,
    height: usize,
    _vertical: bool,
    _barrier_w: usize,
    old_side: &[bool],
    new_side: &[bool],
    global: &LandGlobalConfig,
    rng: &mut ChaCha12Rng,
) {
    let island_max = dynamic_island_max(&size_from_dims(width, height), global);
    let mid_max = dynamic_mid_max(&size_from_dims(width, height), global);

    let mut analysis =
        analyze_landmask(grid, width, height, island_max, mid_max, global.min_lake_size);
    let mut old_comp = dominant_component_on_mask(&analysis, old_side);
    let mut new_comp = dominant_component_on_mask(&analysis, new_side);

    // If the new-world side has no continent, seed one.
    if new_comp.is_none() || new_comp == old_comp {
        seed_new_world_component(grid, width, height, new_side, rng);
        analysis = analyze_landmask(grid, width, height, island_max, mid_max, global.min_lake_size);
        old_comp = dominant_component_on_mask(&analysis, old_side);
        new_comp = dominant_component_on_mask(&analysis, new_side);
    }

    let Some(old_id) = old_comp else {
        return;
    };
    let Some(new_id) = new_comp else {
        return;
    };
    if old_id == new_id {
        return;
    }

    for (idx, &cid) in analysis.land_component_ids.iter().enumerate() {
        if cid == usize::MAX {
            continue;
        }
        if cid != old_id && cid != new_id {
            grid[idx] = 0;
        }
    }
}

/// Pick the dominant component overlapping a side mask.
fn dominant_component_on_mask(analysis: &LandAnalysis, side_mask: &[bool]) -> Option<usize> {
    if analysis.land_component_sizes.is_empty() {
        return None;
    }

    let mut overlap = vec![0usize; analysis.land_component_sizes.len()];
    for (idx, &cid) in analysis.land_component_ids.iter().enumerate() {
        if cid == usize::MAX || !side_mask[idx] {
            continue;
        }
        overlap[cid] += 1;
    }

    overlap
        .into_iter()
        .enumerate()
        .max_by_key(|(_, count)| *count)
        .and_then(|(cid, count)| if count > 0 { Some(cid) } else { None })
}

/// Create a deterministic seed blob on the new-world side if that side is empty.
fn seed_new_world_component(
    grid: &mut [u8],
    width: usize,
    height: usize,
    new_side: &[bool],
    rng: &mut ChaCha12Rng,
) {
    let mut candidates = Vec::new();
    for y in 1..height.saturating_sub(1) {
        for x in 1..width.saturating_sub(1) {
            let idx = y * width + x;
            if new_side[idx] && grid[idx] == 0 {
                candidates.push((rng.next_u64(), idx));
            }
        }
    }
    if candidates.is_empty() {
        return;
    }
    candidates.sort_unstable_by_key(|v| v.0);
    let center_idx = candidates[0].1;
    let blob_size = ((width * height) / 20).clamp(30, 220);
    grow_blob_from_center(grid, width, height, center_idx, 1, blob_size, rng);
}

/// Build a synthetic map-size enum from dimensions for dynamic thresholds.
fn size_from_dims(width: usize, height: usize) -> MapSizes {
    match (width, height) {
        (44, 26) => MapSizes::Duel,
        (60, 38) => MapSizes::Tiny,
        (74, 46) => MapSizes::Small,
        (84, 54) => MapSizes::Standard,
        (96, 60) => MapSizes::Large,
        (106, 66) => MapSizes::Huge,
        _ => MapSizes::Standard,
    }
}

/// Compute inland distance from each land tile to ocean using BFS.
fn inland_distance_to_ocean(grid: &[u8], ocean_mask: &[bool], width: usize, height: usize) -> Vec<u16> {
    let mut dist = vec![u16::MAX; width * height];
    let mut q = VecDeque::new();

    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            if grid[idx] != 1 {
                continue;
            }

            let coastal = neighbors_odd_r(x, y, width, height)
                .into_iter()
                .any(|(nx, ny)| ocean_mask[ny * width + nx]);
            if coastal {
                dist[idx] = 0;
                q.push_back((x, y));
            }
        }
    }

    while let Some((x, y)) = q.pop_front() {
        let idx = y * width + x;
        let d = dist[idx];
        for (nx, ny) in neighbors_odd_r(x, y, width, height) {
            let nidx = ny * width + nx;
            if grid[nidx] != 1 || dist[nidx] <= d + 1 {
                continue;
            }
            dist[nidx] = d + 1;
            q.push_back((nx, ny));
        }
    }

    dist
}

/// Grow a connected blob from a center tile, using deterministic RNG-based frontier ordering.
fn grow_blob_from_center(
    grid: &mut [u8],
    width: usize,
    height: usize,
    center_idx: usize,
    value: u8,
    max_tiles: usize,
    rng: &mut ChaCha12Rng,
) {
    let mut frontier = VecDeque::new();
    let mut visited = vec![false; grid.len()];

    let cx = center_idx % width;
    let cy = center_idx / width;

    frontier.push_back((cx, cy));
    visited[center_idx] = true;

    let mut changed = 0usize;
    while let Some((x, y)) = frontier.pop_front() {
        let idx = y * width + x;
        if x == 0 || y == 0 || x + 1 == width || y + 1 == height {
            continue;
        }

        grid[idx] = value;
        changed += 1;
        if changed >= max_tiles {
            break;
        }

        let mut neighbors = neighbors_odd_r(x, y, width, height)
            .into_iter()
            .map(|(nx, ny)| (rng.next_u64(), nx, ny))
            .collect::<Vec<_>>();
        neighbors.sort_unstable_by_key(|n| n.0);

        for (_, nx, ny) in neighbors {
            let nidx = ny * width + nx;
            if visited[nidx] {
                continue;
            }
            visited[nidx] = true;
            frontier.push_back((nx, ny));
        }
    }
}

/// Compute dynamic island threshold from map size and global config.
fn dynamic_island_max(size: &MapSizes, global: &LandGlobalConfig) -> usize {
    (size.grid_size() / global.island_max_divisor.max(1)).clamp(global.island_max_min, global.island_max_max)
}

/// Compute dynamic mid-size threshold from map size and global config.
fn dynamic_mid_max(size: &MapSizes, global: &LandGlobalConfig) -> usize {
    (size.grid_size() / global.mid_max_divisor.max(1)).clamp(global.mid_max_min, global.mid_max_max)
}

/// Force water on all map borders.
fn enforce_border_water(grid: &mut [u8], width: usize, height: usize) {
    for x in 0..width {
        grid[x] = 0;
        grid[(height - 1) * width + x] = 0;
    }
    for y in 0..height {
        grid[y * width] = 0;
        grid[y * width + (width - 1)] = 0;
    }
}

/// Mirror half-map into full-map along vertical axis.
fn mirror_vertical_into(half: &[u8], out: &mut [u8], width: usize, height: usize) {
    let half_w = width.div_ceil(2);
    for y in 0..height {
        for x in 0..width {
            let src_x = if x < half_w { x } else { width - 1 - x };
            out[y * width + x] = half[y * half_w + src_x];
        }
    }
}

/// Enforce exact vertical symmetry in-place.
fn enforce_vertical_mirror(grid: &mut [u8], width: usize, height: usize) {
    let half_w = width.div_ceil(2);
    for y in 0..height {
        for x in 0..half_w {
            let v = grid[y * width + x];
            let mx = width - 1 - x;
            grid[y * width + mx] = v;
        }
    }
}

/// Compute center tile of one component using component IDs.
fn component_center(analysis: &LandAnalysis, component_id: usize, width: usize) -> (usize, usize) {
    let mut sx = 0usize;
    let mut sy = 0usize;
    let mut n = 0usize;

    for (idx, &cid) in analysis.land_component_ids.iter().enumerate() {
        if cid != component_id {
            continue;
        }
        sx += idx % width;
        sy += idx / width;
        n += 1;
    }

    if n == 0 { (0, 0) } else { (sx / n, sy / n) }
}

/// Draw a soft-width straight land bridge between two points.
fn draw_soft_line(
    grid: &mut [u8],
    width: usize,
    height: usize,
    from: (usize, usize),
    to: (usize, usize),
    radius: usize,
) {
    let dx = to.0 as isize - from.0 as isize;
    let dy = to.1 as isize - from.1 as isize;
    let steps = dx.unsigned_abs().max(dy.unsigned_abs()).max(1);

    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let x = (from.0 as f32 + dx as f32 * t).round() as isize;
        let y = (from.1 as f32 + dy as f32 * t).round() as isize;

        for ry in -(radius as isize)..=(radius as isize) {
            for rx in -(radius as isize)..=(radius as isize) {
                let nx = x + rx;
                let ny = y + ry;
                if nx <= 0 || ny <= 0 || nx >= width as isize - 1 || ny >= height as isize - 1 {
                    continue;
                }
                grid[ny as usize * width + nx as usize] = 1;
            }
        }
    }
}

/// Hex distance helper (offset odd-r -> cube conversion).
fn hex_distance_offset(a: (usize, usize), b: (usize, usize)) -> usize {
    let ac = oddr_to_cube(a.0 as i32, a.1 as i32);
    let bc = oddr_to_cube(b.0 as i32, b.1 as i32);
    ((ac.0 - bc.0)
        .abs()
        .max((ac.1 - bc.1).abs())
        .max((ac.2 - bc.2).abs())) as usize
}

/// Convert odd-r offset hex coordinates to cube coordinates.
fn oddr_to_cube(col: i32, row: i32) -> (i32, i32, i32) {
    let x = col - (row - (row & 1)) / 2;
    let z = row;
    let y = -x - z;
    (x, y, z)
}
