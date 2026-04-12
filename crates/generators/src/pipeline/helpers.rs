use serde::Deserialize;
use std::{fs, path::PathBuf, sync::OnceLock};

#[derive(Debug, Clone, Deserialize)]
/// Config for the biome settings loaded from `biomes.yaml`.
pub struct BiomesConfig {
    pub terrain: TerrainThresholds,
    pub temperature: TemperatureConfig,
    pub rainfall: NoiseConfig,
    pub heightmap: NoiseConfig,
}

#[derive(Debug, Clone, Deserialize)]
/// Terrain thresholds used when converting climate and elevation into base terrain.
pub struct TerrainThresholds {
    pub mountain_threshold: f32,
    pub hill_threshold: f32,
    pub snow_temp_threshold: u8,
    pub tundra_temp_threshold: u8,
    pub desert_temp_threshold: u8,
    pub desert_rain_threshold: u8,
    pub grassland_rain_threshold: u8,
}

#[derive(Debug, Clone, Deserialize)]
/// Temperature noise settings loaded from `biomes.yaml`.
pub struct TemperatureConfig {
    pub continental_octaves: usize,
    pub continental_scale: f64,
    pub detail_octaves: usize,
    pub detail_scale: f64,
    pub continental_weight: f64,
    pub detail_weight: f64,
    pub base_amplitude: f64,
    pub latitude_amp_floor: f64,
}

#[derive(Debug, Clone, Deserialize)]
/// Generic FBM noise settings.
pub struct NoiseConfig {
    pub octaves: usize,
    pub scale: f64,
}

#[derive(Debug, Clone, Deserialize)]
/// Root config for land generation loaded from `landmasses.yml`.
pub struct LandmassesConfig {
    pub global: LandGlobalConfig,
    pub continents: LandStyleConfig,
    pub small_continents: LandStyleConfig,
    pub island_continents: LandStyleConfig,
    pub pangea: LandStyleConfig,
    pub terra: TerraConfig,
    pub mirror: MirrorConfig,
}

#[derive(Debug, Clone, Deserialize)]
/// Shared global settings used by all map styles.
pub struct LandGlobalConfig {
    pub base_factor: usize,
    pub max_repair_iters: usize,
    pub min_lake_size: usize,
    pub island_max_min: usize,
    pub island_max_max: usize,
    pub island_max_divisor: usize,
    pub mid_max_min: usize,
    pub mid_max_max: usize,
    pub mid_max_divisor: usize,
}

#[derive(Debug, Clone, Deserialize)]
/// Draft + constraints + repair knobs for a map style.
pub struct LandStyleConfig {
    pub draft: DraftConfig,
    pub constraints: ConstraintsConfig,
    pub repair: RepairConfig,
}

#[derive(Debug, Clone, Deserialize)]
/// First-pass generation controls before analysis/repair.
pub struct DraftConfig {
    pub base_land_percent: u32,
    pub fuzzy_flip_percent: u32,
    pub coast_island_percent: u32,
    pub smoothing_passes: usize,
    pub center_bias: f32,
}

#[derive(Debug, Clone, Deserialize)]
/// Acceptance constraints for analyze/repair.
pub struct ConstraintsConfig {
    pub min_land_ratio: f32,
    pub max_land_ratio: f32,
    pub min_largest_ratio: f32,
    pub max_largest_ratio: f32,
    pub min_components: usize,
    pub max_components: usize,
    pub min_islands: usize,
    pub min_lakes: usize,
    pub max_lakes: usize,
}

#[derive(Debug, Clone, Deserialize)]
/// Repair behavior knobs used by the deterministic post-processing pass.
pub struct RepairConfig {
    pub largest_carve_trigger_ratio: f32,
    pub largest_carve_target_ratio: f32,
    pub largest_carve_scale: f32,
    pub largest_carve_base_count: usize,
    pub channel_carve_count: usize,
    pub island_min_blob: usize,
    pub island_max_blob: usize,
    pub island_extra_missing_floor: usize,
    pub erode_cap_ratio: f32,
    pub pangea_fill_internal_count: usize,
    pub pangea_connect_count: usize,
    pub pangea_connect_when_split: usize,
    pub terra_grow_budget: usize,
    pub land_ratio_adjust_cap_divisor: usize,
    pub lake_blob_min: usize,
    pub lake_blob_max: usize,
}

#[derive(Debug, Clone, Deserialize)]
/// Terra settings for ocean barrier and split-world generation.
pub struct TerraConfig {
    pub old_world: LandStyleConfig,
    pub new_world: LandStyleConfig,
    pub merged_constraints: ConstraintsConfig,
    pub merged_repair: RepairConfig,
    pub barrier_min: usize,
    pub barrier_max: usize,
}

#[derive(Debug, Clone, Deserialize)]
/// Mirror settings where the right side is reflected from the left side.
pub struct MirrorConfig {
    pub base: LandStyleConfig,
    pub half_smoothing_passes: usize,
}

/// Default biome config used when `biomes.yaml` is not available.
pub fn default_biomes_config() -> BiomesConfig {
    BiomesConfig {
        terrain: TerrainThresholds {
            mountain_threshold: 0.05,
            hill_threshold: 0.2,
            snow_temp_threshold: 40,
            tundra_temp_threshold: 85,
            desert_temp_threshold: 150,
            desert_rain_threshold: 85,
            grassland_rain_threshold: 155,
        },
        temperature: TemperatureConfig {
            continental_octaves: 4,
            continental_scale: 120.0,
            detail_octaves: 5,
            detail_scale: 35.0,
            continental_weight: 0.7,
            detail_weight: 0.3,
            base_amplitude: 0.18,
            latitude_amp_floor: 0.5,
        },
        rainfall: NoiseConfig {
            octaves: 5,
            scale: 60.0,
        },
        heightmap: NoiseConfig {
            octaves: 5,
            scale: 40.0,
        },
    }
}

fn default_style(
    base_land_percent: u32,
    fuzzy_flip_percent: u32,
    coast_island_percent: u32,
    smoothing_passes: usize,
    center_bias: f32,
    constraints: ConstraintsConfig,
) -> LandStyleConfig {
    LandStyleConfig {
        draft: DraftConfig {
            base_land_percent,
            fuzzy_flip_percent,
            coast_island_percent,
            smoothing_passes,
            center_bias,
        },
        constraints,
        repair: RepairConfig {
            largest_carve_trigger_ratio: 0.65,
            largest_carve_target_ratio: 0.55,
            largest_carve_scale: 30.0,
            largest_carve_base_count: 2,
            channel_carve_count: 6,
            island_min_blob: 2,
            island_max_blob: 6,
            island_extra_missing_floor: 2,
            erode_cap_ratio: 0.30,
            pangea_fill_internal_count: 12,
            pangea_connect_count: 3,
            pangea_connect_when_split: 2,
            terra_grow_budget: 40,
            land_ratio_adjust_cap_divisor: 10,
            lake_blob_min: 4,
            lake_blob_max: 7,
        },
    }
}

/// Default landmass config used when `landmasses.yml` is not available.
pub fn default_landmasses_config() -> LandmassesConfig {
    let continents_constraints = ConstraintsConfig {
        min_land_ratio: 0.35,
        max_land_ratio: 0.55,
        min_largest_ratio: 0.25,
        max_largest_ratio: 0.55,
        min_components: 2,
        max_components: 6,
        min_islands: 2,
        min_lakes: 1,
        max_lakes: 4,
    };

    let small_constraints = ConstraintsConfig {
        min_land_ratio: 0.30,
        max_land_ratio: 0.50,
        min_largest_ratio: 0.0,
        max_largest_ratio: 0.45,
        min_components: 5,
        max_components: 15,
        min_islands: 6,
        min_lakes: 1,
        max_lakes: 6,
    };

    let island_constraints = ConstraintsConfig {
        min_land_ratio: 0.20,
        max_land_ratio: 0.40,
        min_largest_ratio: 0.0,
        max_largest_ratio: 0.30,
        min_components: 8,
        max_components: 32,
        min_islands: 12,
        min_lakes: 0,
        max_lakes: 3,
    };

    let pangea_constraints = ConstraintsConfig {
        min_land_ratio: 0.35,
        max_land_ratio: 0.55,
        min_largest_ratio: 0.80,
        max_largest_ratio: 1.0,
        min_components: 1,
        max_components: 4,
        min_islands: 1,
        min_lakes: 1,
        max_lakes: 6,
    };

    let terra_merged_constraints = ConstraintsConfig {
        min_land_ratio: 0.35,
        max_land_ratio: 0.55,
        min_largest_ratio: 0.45,
        max_largest_ratio: 0.70,
        min_components: 2,
        max_components: 10,
        min_islands: 2,
        min_lakes: 1,
        max_lakes: 4,
    };

    let mirror_constraints = ConstraintsConfig {
        min_land_ratio: 0.35,
        max_land_ratio: 0.55,
        min_largest_ratio: 0.25,
        max_largest_ratio: 0.60,
        min_components: 2,
        max_components: 12,
        min_islands: 2,
        min_lakes: 0,
        max_lakes: 5,
    };

    LandmassesConfig {
        global: LandGlobalConfig {
            base_factor: 16,
            max_repair_iters: 4,
            min_lake_size: 4,
            island_max_min: 20,
            island_max_max: 40,
            island_max_divisor: 220,
            mid_max_min: 120,
            mid_max_max: 260,
            mid_max_divisor: 28,
        },
        continents: default_style(9, 7, 5, 2, 0.0, continents_constraints),
        small_continents: default_style(8, 12, 8, 1, 0.0, small_constraints),
        island_continents: default_style(6, 14, 12, 0, 0.0, island_constraints),
        pangea: default_style(10, 4, 2, 2, 0.65, pangea_constraints),
        terra: TerraConfig {
            old_world: default_style(11, 6, 4, 2, 0.30, terra_merged_constraints.clone()),
            new_world: default_style(8, 10, 8, 1, 0.15, terra_merged_constraints.clone()),
            merged_constraints: terra_merged_constraints,
            merged_repair: RepairConfig {
                largest_carve_trigger_ratio: 1.0,
                largest_carve_target_ratio: 1.0,
                largest_carve_scale: 0.0,
                largest_carve_base_count: 0,
                channel_carve_count: 0,
                island_min_blob: 2,
                island_max_blob: 5,
                island_extra_missing_floor: 2,
                erode_cap_ratio: 1.0,
                pangea_fill_internal_count: 0,
                pangea_connect_count: 0,
                pangea_connect_when_split: 0,
                terra_grow_budget: 40,
                land_ratio_adjust_cap_divisor: 10,
                lake_blob_min: 4,
                lake_blob_max: 7,
            },
            barrier_min: 6,
            barrier_max: 12,
        },
        mirror: MirrorConfig {
            base: default_style(9, 9, 5, 1, 0.0, mirror_constraints),
            half_smoothing_passes: 2,
        },
    }
}

/// Location of `biomes.yaml`.
pub fn biomes_config_path() -> PathBuf {
    if let Ok(path) = std::env::var("CIVORUM_BIOMES_CONFIG") {
        return PathBuf::from(path);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../biomes.yaml")
}

/// Location of `landmasses.yml`.
pub fn landmasses_config_path() -> PathBuf {
    if let Ok(path) = std::env::var("CIVORUM_LANDMASSES_CONFIG") {
        return PathBuf::from(path);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../landmasses.yml")
}

/// Load and parse biome config from yaml.
pub fn load_biomes_config() -> BiomesConfig {
    let path = biomes_config_path();
    match fs::read_to_string(&path) {
        Ok(raw) => match serde_yaml::from_str::<BiomesConfig>(&raw) {
            Ok(config) => config,
            Err(err) => {
                eprintln!(
                    "Failed to parse biome config at '{}': {err}. Falling back to defaults.",
                    path.display()
                );
                default_biomes_config()
            }
        },
        Err(err) => {
            eprintln!(
                "Failed to read biome config at '{}': {err}. Falling back to defaults.",
                path.display()
            );
            default_biomes_config()
        }
    }
}

/// Load and parse landmass config from yaml.
pub fn load_landmasses_config() -> LandmassesConfig {
    let path = landmasses_config_path();
    match fs::read_to_string(&path) {
        Ok(raw) => match serde_yaml::from_str::<LandmassesConfig>(&raw) {
            Ok(config) => config,
            Err(err) => {
                eprintln!(
                    "Failed to parse landmass config at '{}': {err}. Falling back to defaults.",
                    path.display()
                );
                default_landmasses_config()
            }
        },
        Err(err) => {
            eprintln!(
                "Failed to read landmass config at '{}': {err}. Falling back to defaults.",
                path.display()
            );
            default_landmasses_config()
        }
    }
}

/// Cached biome config singleton.
pub fn biomes_config() -> &'static BiomesConfig {
    static CONFIG: OnceLock<BiomesConfig> = OnceLock::new();
    CONFIG.get_or_init(load_biomes_config)
}

/// Cached landmass config singleton.
pub fn landmasses_config() -> &'static LandmassesConfig {
    static CONFIG: OnceLock<LandmassesConfig> = OnceLock::new();
    CONFIG.get_or_init(load_landmasses_config)
}

/// Helper function for odd-r neighbors for pointy-top hexes.
/// Returns only in-bounds neighbors.
pub fn neighbors_odd_r(x: usize, y: usize, width: usize, height: usize) -> Vec<(usize, usize)> {
    let p = y & 1;

    let x = x as isize;
    let y = y as isize;
    let width = width as isize;
    let height = height as isize;

    let candidates: [(isize, isize); 6] = if p == 0 {
        [
            (x, y - 1),
            (x + 1, y),
            (x, y + 1),
            (x - 1, y + 1),
            (x - 1, y),
            (x - 1, y - 1),
        ]
    } else {
        [
            (x + 1, y - 1),
            (x + 1, y),
            (x + 1, y + 1),
            (x, y + 1),
            (x - 1, y),
            (x, y - 1),
        ]
    };

    let mut out = Vec::with_capacity(6);

    for (nx, ny) in candidates {
        if ny < 0 || ny >= height {
            continue;
        }
        if nx < 0 || nx >= width {
            continue;
        }
        out.push((nx as usize, ny as usize));
    }

    out
}
