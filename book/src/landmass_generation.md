# Landmass Generation

Civorum land generation is deterministic per seed and map type.

The pipeline works in three stages:

1. Draft generation: coarse seed, zoom refinement, and smoothing.
2. Analysis: measure ratios and connected components.
3. Repair: deterministic operations (straits, channels, islands, lakes, growth/shrink) to satisfy targets.

All tuning lives in `landmasses.yml`.

## Map Types

### Continents
Goal: a few large continents plus some islands and lakes.

Typical behavior:
- 2-6 land components
- moderate largest component share
- moderate coastline complexity

Best knobs:
- `continents.draft.base_land_percent`: raises/lowers initial land amount.
- `continents.constraints.max_largest_ratio`: lowers supercontinent risk.
- `continents.repair.largest_carve_*`: controls how aggressively large blobs are split.
- `continents.constraints.min_components`: minimum number of separate landmasses.

### Small Continents
Goal: many separated landmasses with no dominant supercontinent.

Typical behavior:
- higher fragmentation than continents
- stronger channel carving

Best knobs:
- `small_continents.repair.channel_carve_count`: raises split frequency.
- `small_continents.constraints.max_largest_ratio`: caps dominance.
- `small_continents.draft.fuzzy_flip_percent`: rougher coastlines and more fragmentation.

### Island Continents
Goal: archipelago-like world with many islands and no very large continent.

Typical behavior:
- many small/mid components
- low to mid land ratio

Best knobs:
- `island_continents.draft.coast_island_percent`: creates more island emergence during zoom.
- `island_continents.repair.erode_cap_ratio`: erodes oversized components.
- `island_continents.constraints.min_islands`: forces island count floor.

### Pangea
Goal: one dominant landmass with a few offshore islands and some lakes.

Typical behavior:
- very high largest component share
- optional minor off-continent fragments

Best knobs:
- `pangea.draft.center_bias`: concentrates land toward a core.
- `pangea.constraints.min_largest_ratio`: forces dominance.
- `pangea.repair.pangea_fill_internal_count`: fills internal channels.
- `pangea.repair.pangea_connect_count`: reconnects detached components.

### Terra
Goal: old world and new world separated by a guaranteed ocean barrier.

Typical behavior:
- split-world layout
- old/new sides can use different draft settings

Best knobs:
- `terra.barrier_min` / `terra.barrier_max`: guaranteed ocean band width.
- `terra.old_world.*` and `terra.new_world.*`: side-specific tuning.
- `terra.merged_constraints.*`: final combined-world acceptance targets.
- `terra.merged_repair.terra_grow_budget`: growth budget if second cluster is too small.

### Mirror
Goal: exact vertical symmetry with natural-looking local structure.

Typical behavior:
- left half generated and smoothed, then mirrored
- symmetry preserved after repair

Best knobs:
- `mirror.half_smoothing_passes`: smoothness before reflection.
- `mirror.base.constraints.*`: overall ratio/component targets.
- `mirror.base.repair.largest_carve_*`: anti-supercontinent control in symmetric mode.

## `landmasses.yml` Variable Reference

## `global`

- `base_factor`: coarse draft grid divisor. Higher means coarser first pass and stronger large-scale structure.
- `max_repair_iters`: number of analyze/repair iterations before final hard enforcement.
- `min_lake_size`: minimum enclosed water component size counted as a lake.
- `island_max_min`: lower bound for dynamic island size threshold.
- `island_max_max`: upper bound for dynamic island size threshold.
- `island_max_divisor`: map-size divisor used to compute dynamic island threshold.
- `mid_max_min`: lower bound for dynamic mid-size threshold.
- `mid_max_max`: upper bound for dynamic mid-size threshold.
- `mid_max_divisor`: map-size divisor used to compute dynamic mid-size threshold.

## `*.draft`
(Used in each map type style block: `continents`, `small_continents`, `island_continents`, `pangea`, `terra.old_world`, `terra.new_world`, `mirror.base`.)

- `base_land_percent`: baseline probability of land in coarse draft stage.
- `fuzzy_flip_percent`: chance to flip mixed parent tiles during zoom; raises coastline noise.
- `coast_island_percent`: chance to spawn small land on mixed coast transitions during zoom.
- `smoothing_passes`: hex-neighborhood smoothing passes after zoom.
- `center_bias`: land attraction toward a random center. Higher values promote large central masses.

## `*.constraints`

- `min_land_ratio`: minimum final land share.
- `max_land_ratio`: maximum final land share.
- `min_largest_ratio`: minimum share of total land held by largest component.
- `max_largest_ratio`: maximum share of total land held by largest component.
- `min_components`: minimum number of land components.
- `max_components`: maximum number of land components.
- `min_islands`: minimum number of island-like components.
- `min_lakes`: minimum number of lakes.
- `max_lakes`: maximum number of lakes.

## `*.repair`

- `largest_carve_trigger_ratio`: start carving straits when largest component ratio exceeds this value.
- `largest_carve_target_ratio`: target largest ratio used to compute carve intensity.
- `largest_carve_scale`: carve intensity multiplier from largest-ratio overshoot.
- `largest_carve_base_count`: base number of strait carve edits.
- `channel_carve_count`: targeted channel carving pass size.
- `island_min_blob`: minimum tiles when sprinkling new islands.
- `island_max_blob`: maximum tiles when sprinkling new islands.
- `island_extra_missing_floor`: minimum island additions when island count is below target.
- `erode_cap_ratio`: largest-component cap for erosion-heavy styles (mainly island continents).
- `pangea_fill_internal_count`: number of narrow internal channels to fill.
- `pangea_connect_count`: number of detached components to connect back to the largest one.
- `pangea_connect_when_split`: extra reconnect budget when component count is too high.
- `terra_grow_budget`: growth budget used when the second major cluster is undersized.
- `land_ratio_adjust_cap_divisor`: limits per-iteration ratio correction budget.
- `lake_blob_min`: minimum carved lake size.
- `lake_blob_max`: maximum carved lake size.

## `terra`

- `old_world`: style config for one side of the split map.
- `new_world`: style config for the opposite side.
- `merged_constraints`: constraints applied after both sides are combined.
- `merged_repair`: repair settings for the combined map.
- `barrier_min`: minimum ocean barrier width.
- `barrier_max`: maximum ocean barrier width.

## `mirror`

- `base`: style config used for half-map generation before reflection.
- `half_smoothing_passes`: smoothing passes applied on half-map before mirroring.

## Practical Tuning Advice

1. Start with constraints, then tune draft, then tune repair.
2. If maps are too blob-like on larger sizes, increase `largest_carve_base_count`, `largest_carve_scale`, or `channel_carve_count`.
3. If coastlines look noisy, reduce `fuzzy_flip_percent` and/or increase `smoothing_passes`.
4. If island counts are low, increase `coast_island_percent` and `island_extra_missing_floor`.
5. If maps meet shape goals but miss area goals, adjust `min_land_ratio` and `max_land_ratio` first, then rebalance `base_land_percent`.
